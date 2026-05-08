pub use {
    icondata::AiCaretDownFilled as CaretDown, icondata::AiCaretUpFilled as CaretUp,
    icondata::AiCheckOutlined as Accept, icondata::AiCloseOutlined as Close,
    icondata::AiMinusOutlined as Remove, icondata::AiPlusOutlined as Add,
    icondata::AiSearchOutlined as Search, icondata::AiSyncOutlined as Refresh,
    icondata::AiUserOutlined as User, icondata::BsGraphUp as Graph,
    icondata::FaFilterSolid as Filter, icondata::FaFlagRegular as Flag,
    icondata::FaPenSolid as Edit, icondata::FaTrashCanSolid as Trash,
    icondata::IoSettingsSharp as Settings, icondata::LuEye as Eye,
    icondata::TbEyeClosedOutline as EyeClosed, icondata::VsChevronDown as ChevronDown,
    icondata::VsChevronRight as ChevronRight,
};

pub mod workspace {
    pub use {
        icondata::BsGraphUp as Spreadsheet, icondata::ImTree as Graph, icondata::VsBrowser as Db,
    };
}

pub mod file_type {
    use leptos::prelude::*;
    use leptos_icons::Symbol;
    use std::{fmt::Display, path::Path};

    const DEFAULT_ICON: icondata::Icon = icondata::FaFileRegular;
    const ICON_SYMBOL_IDS: &[(icondata::Icon, &str)] = &[
        (icondata::FaFileRegular, "default"),
        (icondata::FaFileAudioRegular, "audio"),
        (icondata::FaPythonBrands, "python"),
        (icondata::FaRProjectBrands, "r"),
        (icondata::FaFileCodeRegular, "code"),
        (icondata::FaFileExcelRegular, "excel"),
        (icondata::FaFileImageRegular, "image"),
        (icondata::FaFileLinesRegular, "txt"),
        (icondata::FaFilePdfRegular, "pdf"),
        (icondata::FaFilePowerpointRegular, "ppt"),
        (icondata::FaFileWordRegular, "doc"),
        (icondata::FaFileVideoRegular, "video"),
        (icondata::FaFileZipperRegular, "zip"),
        (icondata::OcFileBinaryLg, "binary"),
    ];

    pub type Color = &'static str;
    pub struct ThemedColor {
        light: Color,
        dark: Color,
    }

    impl Default for ThemedColor {
        fn default() -> Self {
            Self {
                light: "secondary-900",
                dark: "secondary-50",
            }
        }
    }

    impl Display for ThemedColor {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "text-{} dark:text-{}", self.light, self.dark)
        }
    }

    pub struct IconData {
        icon: icondata::Icon,
        color: ThemedColor,
        symbol_id: &'static str,
    }

    impl IconData {
        /// Get the icon data to display for file type.
        pub fn from_path(path: impl AsRef<Path>) -> Self {
            let Some(extension) = path.as_ref().extension() else {
                return Self::default();
            };

            let Some(extension) = extension.to_str() else {
                return Self::default();
            };

            let icon = icon_from_extension(extension);
            let symbol_id = ICON_SYMBOL_IDS
                .iter()
                .find_map(|(sym_icon, id)| (icon == *sym_icon).then_some(id))
                .unwrap();

            Self {
                icon,
                color: color_from_extension(extension),
                symbol_id,
            }
        }

        pub fn icon(&self) -> &icondata::Icon {
            &self.icon
        }

        pub fn color(&self) -> &ThemedColor {
            &self.color
        }

        pub fn symbol_id(&self) -> &'static str {
            self.symbol_id
        }
    }

    impl Default for IconData {
        fn default() -> Self {
            Self {
                icon: DEFAULT_ICON,
                color: ThemedColor::default(),
                symbol_id: "default",
            }
        }
    }

    pub fn icon_from_extension(ext: impl AsRef<str>) -> icondata::Icon {
        match ext.as_ref() {
            "mp3" | "m4a" | "flac" | "wav" => icondata::FaFileAudioRegular,
            "py" => icondata::FaPythonBrands,
            "r" => icondata::FaRProjectBrands,
            "m" | "js" | "ts" | "cpp" | "c" | "rs" => icondata::FaFileCodeRegular,
            "csv" | "xlsx" | "xlsm" | "xml" | "odf" => icondata::FaFileExcelRegular,
            "png" | "svg" | "jpg" | "jpeg" | "tiff" | "bmp" => icondata::FaFileImageRegular,
            "txt" => icondata::FaFileLinesRegular,
            "pdf" => icondata::FaFilePdfRegular,
            "pptx" | "pptm" | "ppt" => icondata::FaFilePowerpointRegular,
            "doc" | "docm" | "docx" | "dot" => icondata::FaFileWordRegular,
            "mp4" | "mov" | "wmv" | "avi" => icondata::FaFileVideoRegular,
            "zip" | "zipx" | "rar" | "7z" | "gz" => icondata::FaFileZipperRegular,
            "dat" | "pkl" | "bin" | "exe" => icondata::OcFileBinaryLg,
            _ => DEFAULT_ICON,
        }
    }

    pub fn color_from_extension(ext: impl AsRef<str>) -> ThemedColor {
        let (light, dark) = match ext.as_ref() {
            "mp3" | "m4a" | "flac" | "wav" => ("syre-yellow-800", "syre-yellow-500"),
            "py" | "r" | "m" | "js" | "ts" | "cpp" | "c" | "rs" => ("primary-700", "primary-400"),
            "csv" | "xlsx" | "xlsm" | "xml" | "odf" => ("syre-green-700", "syre-green-400"),
            "png" | "svg" | "jpg" | "jpeg" | "tiff" | "bmp" => {
                ("syre-yellow-700", "syre-yellow-400")
            }
            "txt" => ("primary-800", "primary-100"),
            "pdf" => ("syre-red-800", "syre-red-500"),
            "pptx" | "pptm" | "ppt" => ("syre-red-700", "syre-red-400"),
            "doc" | "docm" | "docx" | "dot" => ("primary-800", "primary-500"),
            "mp4" | "mov" | "wmv" | "avi" => ("syre-yellow-900", "syre-yellow-600"),
            "zip" | "zipx" | "rar" | "7z" | "gz" => ("primary-900", "primary-200"),
            "dat" | "pkl" | "bin" | "exe" => ("syre-green-900", "syre-green-200"),
            _ => ("secondary-900", "secondary-50"),
        };

        ThemedColor { light, dark }
    }

    /// Get the icon id to display for file type.
    pub fn icon(path: impl AsRef<Path>) -> icondata::Icon {
        let Some(extension) = path.as_ref().extension() else {
            return DEFAULT_ICON;
        };
        let Some(extension) = extension.to_str() else {
            return DEFAULT_ICON;
        };

        icon_from_extension(extension)
    }

    /// Get the icon color to display for file type.
    ///
    /// # Returns
    /// Color class for the file type icon.
    pub fn icon_color(path: impl AsRef<Path>) -> ThemedColor {
        let Some(extension) = path.as_ref().extension() else {
            return ThemedColor::default();
        };
        let Some(extension) = extension.to_str() else {
            return ThemedColor::default();
        };

        color_from_extension(extension)
    }

    /// Display all possible file type icons as `<symbol>`s.
    #[component]
    pub fn IconSymbols(prefix: &'static str) -> impl IntoView {
        ICON_SYMBOL_IDS
            .iter()
            .map(|(icon, id)| view! { <Symbol id=format!("{prefix}-{id}") icon=*icon /> })
            .collect::<Vec<_>>()
    }
}
