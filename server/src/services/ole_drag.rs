use std::mem::{size_of, ManuallyDrop};
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows::core::{implement, Error, HRESULT, Result};
use windows::Win32::Foundation::{
    BOOL, DATA_S_SAMEFORMATETC, DRAGDROP_S_CANCEL, DRAGDROP_S_DROP, DRAGDROP_S_USEDEFAULTCURSORS,
    DV_E_FORMATETC, E_INVALIDARG, E_NOTIMPL, OLE_E_ADVISENOTSUPPORTED, OLE_E_NOCONNECTION, POINT,
    RPC_E_CHANGED_MODE, S_FALSE,
};
use windows::Win32::System::Com::{
    IAdviseSink, IAgileObject, IAgileObject_Impl, IDataObject, IDataObject_Impl, IEnumFORMATETC,
    IEnumFORMATETC_Impl, IEnumSTATDATA, DATADIR_GET, DVASPECT_CONTENT, FORMATETC, STGMEDIUM,
    STGMEDIUM_0, TYMED_HGLOBAL,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Ole::{
    DoDragDrop, IDropSource, IDropSource_Impl, OleInitialize, OleUninitialize, CF_HDROP, DROPEFFECT,
    DROPEFFECT_COPY,
};
use windows::Win32::System::SystemServices::{MK_LBUTTON, MODIFIERKEYS_FLAGS};
use windows::Win32::UI::Shell::DROPFILES;

/// 发起一次剪贴板之外的 OLE 文件拖拽。
///
/// 数据对象是自己实现的 CF_HDROP，并声明 IAgileObject：资源管理器/桌面在 Drop
/// 里会同步回调数据对象取文件，若回调必须排回正卡在 DoDragDrop 里的线程就会死锁。
/// agile 对象允许 COM 直接在任意线程上取数据，拖到桌面也能正常落盘。
///
/// 只允许 COPY：置物架里的图片是库文件，拖出去必须留一份原件；
/// 允许 MOVE 时资源管理器会默认执行移动，把图从库里拖走。
pub fn drag_files(paths: &[PathBuf]) -> Result<()> {
    let _apartment = init_apartment()?;
    let data_object: IDataObject = FileDataObject {
        paths: paths.to_vec(),
    }
    .into();
    let drop_source: IDropSource = DragSource.into();

    let mut effect = DROPEFFECT::default();
    let _ = unsafe { DoDragDrop(&data_object, &drop_source, DROPEFFECT_COPY, &mut effect) };
    Ok(())
}

struct Apartment;

impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { OleUninitialize() };
    }
}

fn init_apartment() -> Result<Option<Apartment>> {
    unsafe {
        match OleInitialize(Some(std::ptr::null_mut())) {
            Ok(()) => Ok(Some(Apartment)),
            // 线程已被别处初始化时不再重复初始化，拖拽仍可进行。
            Err(err) if err.code() == RPC_E_CHANGED_MODE => Ok(None),
            Err(err) => Err(err),
        }
    }
}

#[implement(IDataObject, IAgileObject)]
struct FileDataObject {
    paths: Vec<PathBuf>,
}

impl FileDataObject {
    fn accepts(&self, format: &FORMATETC) -> bool {
        format.cfFormat == CF_HDROP.0
            && format.dwAspect == DVASPECT_CONTENT.0
            && format.lindex == -1
            && format.tymed & (TYMED_HGLOBAL.0 as u32) != 0
    }

    fn hdrop_medium(&self) -> Result<STGMEDIUM> {
        let mut names: Vec<u16> = Vec::new();
        for path in &self.paths {
            names.extend(path.as_os_str().encode_wide());
            names.push(0);
        }
        // CF_HDROP 的文件列表以空字符串结尾，即连续两个 0。
        names.push(0);

        let header = DROPFILES {
            pFiles: size_of::<DROPFILES>() as u32,
            pt: POINT::default(),
            fNC: BOOL(0),
            fWide: BOOL(1),
        };
        let bytes = size_of::<DROPFILES>() + names.len() * size_of::<u16>();

        unsafe {
            let block = GlobalAlloc(GMEM_MOVEABLE, bytes)?;
            let target = GlobalLock(block);
            if target.is_null() {
                return Err(Error::new(E_NOTIMPL, "无法锁定拖拽内存"));
            }
            std::ptr::copy_nonoverlapping(
                &header as *const DROPFILES as *const u8,
                target as *mut u8,
                size_of::<DROPFILES>(),
            );
            std::ptr::copy_nonoverlapping(
                names.as_ptr() as *const u8,
                (target as *mut u8).add(size_of::<DROPFILES>()),
                names.len() * size_of::<u16>(),
            );
            let _ = GlobalUnlock(block);

            Ok(STGMEDIUM {
                tymed: TYMED_HGLOBAL.0 as u32,
                u: STGMEDIUM_0 { hGlobal: block },
                pUnkForRelease: ManuallyDrop::new(None),
            })
        }
    }
}

#[allow(non_snake_case)]
impl IDataObject_Impl for FileDataObject_Impl {
    fn GetData(&self, pformatetcin: *const FORMATETC) -> Result<STGMEDIUM> {
        let format = unsafe { pformatetcin.as_ref() }
            .ok_or_else(|| Error::new(E_INVALIDARG, "格式为空"))?;
        if !self.accepts(format) {
            return Err(Error::new(DV_E_FORMATETC, "不支持的拖拽格式"));
        }
        self.hdrop_medium()
    }

    fn GetDataHere(&self, _pformatetc: *const FORMATETC, _pmedium: *mut STGMEDIUM) -> Result<()> {
        Err(Error::new(E_NOTIMPL, "不支持就地取数据"))
    }

    fn QueryGetData(&self, pformatetc: *const FORMATETC) -> HRESULT {
        let Some(format) = (unsafe { pformatetc.as_ref() }) else {
            return E_INVALIDARG;
        };
        if self.accepts(format) {
            HRESULT(0)
        } else {
            DV_E_FORMATETC
        }
    }

    fn GetCanonicalFormatEtc(
        &self,
        pformatectin: *const FORMATETC,
        pformatetcout: *mut FORMATETC,
    ) -> HRESULT {
        if pformatectin.is_null() || pformatetcout.is_null() {
            return E_INVALIDARG;
        }
        unsafe {
            pformatetcout.write(*pformatectin);
            (*pformatetcout).ptd = std::ptr::null_mut();
        }
        DATA_S_SAMEFORMATETC
    }

    fn SetData(
        &self,
        _pformatetc: *const FORMATETC,
        _pmedium: *const STGMEDIUM,
        _frelease: BOOL,
    ) -> Result<()> {
        Err(Error::new(E_NOTIMPL, "数据源只读"))
    }

    fn EnumFormatEtc(&self, dwdirection: u32) -> Result<IEnumFORMATETC> {
        if dwdirection != DATADIR_GET.0 as u32 {
            return Err(Error::new(E_NOTIMPL, "只支持读取格式"));
        }
        Ok(FormatEnumerator {
            index: AtomicUsize::new(0),
        }
        .into())
    }

    fn DAdvise(
        &self,
        _pformatetc: *const FORMATETC,
        _advf: u32,
        _padvsink: Option<&IAdviseSink>,
    ) -> Result<u32> {
        Err(Error::new(OLE_E_ADVISENOTSUPPORTED, "不支持数据变更通知"))
    }

    fn DUnadvise(&self, _dwconnection: u32) -> Result<()> {
        Err(Error::new(OLE_E_NOCONNECTION, "没有数据变更连接"))
    }

    fn EnumDAdvise(&self) -> Result<IEnumSTATDATA> {
        Err(Error::new(OLE_E_ADVISENOTSUPPORTED, "不支持数据变更通知"))
    }
}

impl IAgileObject_Impl for FileDataObject_Impl {}

fn hdrop_format() -> FORMATETC {
    FORMATETC {
        cfFormat: CF_HDROP.0,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    }
}

#[implement(IEnumFORMATETC, IAgileObject)]
struct FormatEnumerator {
    index: AtomicUsize,
}

impl IAgileObject_Impl for FormatEnumerator_Impl {}

#[allow(non_snake_case)]
impl IEnumFORMATETC_Impl for FormatEnumerator_Impl {
    fn Next(&self, celt: u32, rgelt: *mut FORMATETC, pceltfetched: *mut u32) -> HRESULT {
        if rgelt.is_null() {
            return E_INVALIDARG;
        }
        if !pceltfetched.is_null() {
            unsafe { *pceltfetched = 0 };
        }
        if celt == 0 {
            return HRESULT(0);
        }
        if self.index.load(Ordering::Relaxed) > 0 {
            return S_FALSE;
        }
        unsafe { rgelt.write(hdrop_format()) };
        self.index.store(1, Ordering::Relaxed);
        if !pceltfetched.is_null() {
            unsafe { *pceltfetched = 1 };
        }
        if celt == 1 {
            HRESULT(0)
        } else {
            S_FALSE
        }
    }

    fn Skip(&self, celt: u32) -> Result<()> {
        if celt == 0 || self.index.load(Ordering::Relaxed) > 0 {
            return Ok(());
        }
        self.index.store(1, Ordering::Relaxed);
        Ok(())
    }

    fn Reset(&self) -> Result<()> {
        self.index.store(0, Ordering::Relaxed);
        Ok(())
    }

    fn Clone(&self) -> Result<IEnumFORMATETC> {
        Ok(FormatEnumerator {
            index: AtomicUsize::new(self.index.load(Ordering::Relaxed)),
        }
        .into())
    }
}

#[implement(IDropSource)]
struct DragSource;

#[allow(non_snake_case)]
impl IDropSource_Impl for DragSource_Impl {
    fn QueryContinueDrag(&self, fescapepressed: BOOL, grfkeystate: MODIFIERKEYS_FLAGS) -> HRESULT {
        if fescapepressed.as_bool() {
            DRAGDROP_S_CANCEL
        } else if (grfkeystate & MK_LBUTTON) == MODIFIERKEYS_FLAGS(0) {
            DRAGDROP_S_DROP
        } else {
            HRESULT(0)
        }
    }

    fn GiveFeedback(&self, _dweffect: DROPEFFECT) -> HRESULT {
        DRAGDROP_S_USEDEFAULTCURSORS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Ole::ReleaseStgMedium;

    fn format_of(cf: u16) -> FORMATETC {
        FORMATETC {
            cfFormat: cf,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_HGLOBAL.0 as u32,
        }
    }

    #[test]
    fn data_object_serves_file_list() {
        let _ = unsafe { OleInitialize(Some(std::ptr::null_mut())) };
        let path = std::env::temp_dir().join("snapcord_drag_probe.txt");
        std::fs::write(&path, b"probe").unwrap();

        let object: IDataObject = FileDataObject {
            paths: vec![path.clone()],
        }
        .into();
        let hdrop = format_of(CF_HDROP.0);

        unsafe {
            assert_eq!(object.QueryGetData(&hdrop), HRESULT(0));
            let mut medium = object.GetData(&hdrop).unwrap();

            let base = GlobalLock(medium.u.hGlobal);
            assert!(!base.is_null());
            let header = &*(base as *const DROPFILES);
            assert!(header.fWide.as_bool());
            let names = std::slice::from_raw_parts(
                (base as *const u8).add(header.pFiles as usize) as *const u16,
                path.as_os_str().encode_wide().count() + 2,
            );
            let expected: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain([0, 0])
                .collect();
            assert_eq!(names, expected.as_slice());
            let _ = GlobalUnlock(medium.u.hGlobal);

            ReleaseStgMedium(&mut medium);
        }
        let _ = std::fs::remove_file(path);
    }
}
