use leptos::prelude::*;
use syre_core::types::ResourceId;
use syre_desktop_ui_lib as ui_lib;

pub mod asset {
    //! Common Asset functionality.

    /// # Returns
    /// Icon associated to a file extension.
    pub fn extension_icon(extension: impl AsRef<str>) -> icondata::Icon {
        match extension.as_ref() {
            "mp3" | "m4a" | "flac" | "wav" => icondata::FaFileAudioRegular,
            "py" | "r" | "m" | "js" | "ts" | "cpp" | "c" | "rs" => icondata::FaFileCodeRegular,
            "csv" | "xlsx" | "xlsm" | "xml" | "odf" => icondata::FaFileExcelRegular,
            "png" | "svg" | "jpg" | "jpeg" | "tiff" | "bmp" => icondata::FaFileImageRegular,
            "txt" => icondata::FaFileLinesRegular,
            "pdf" => icondata::FaFilePdfRegular,
            "pptx" | "pptm" | "ppt" => icondata::FaFilePowerpointRegular,
            "doc" | "docm" | "docx" | "dot" => icondata::FaFileWordRegular,
            "mp4" | "mov" | "wmv" | "avi" => icondata::FaFileVideoRegular,
            "zip" | "zipx" | "rar" | "7z" | "gz" => icondata::FaFileZipperRegular,
            "dat" | "pkl" | "bin" | "exe" => icondata::OcFileBinaryLg,
            _ => icondata::FaFileRegular,
        }
    }
}
