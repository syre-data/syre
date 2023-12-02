pub mod asset {
    //! Common Asset functionality.
    use thot_core::project::Asset;
    use yew_icons::IconId;

    type Color = String;

    /// Gets the name to display for an [`Asset`].
    ///
    /// # Returns
    /// The `name` if set, otherwise the `path`'s file name.
    pub fn asset_display_name(asset: &Asset) -> String {
        if let Some(name) = asset.properties.name.as_ref() {
            name.clone()
        } else {
            asset.path.as_path().to_str().unwrap().to_string()
        }
    }

    /// Gets the icon id to display for an [`Asset`].
    ///
    /// # Returns
    /// The `IconId`.
    pub fn asset_icon_id(asset: &Asset) -> IconId {
        fn get_icon_id(extension: &str) -> IconId {
            match extension {
                "mp3" | "m4a" | "flac" | "wav" => IconId::FontAwesomeRegularFileAudio,
                "py" | "r" | "m" | "js" | "ts" | "cpp" | "c" | "rs" => {
                    IconId::FontAwesomeRegularFileCode
                }
                "csv" | "xlsx" | "xlsm" | "xml" | "odf" => IconId::FontAwesomeRegularFileExcel,
                "png" | "svg" | "jpg" | "jpeg" | "tiff" | "bmp" => {
                    IconId::FontAwesomeRegularFileImage
                }
                "txt" => IconId::FontAwesomeRegularFileLines,
                "pdf" => IconId::FontAwesomeRegularFilePdf,
                "pptx" | "pptm" | "ppt" => IconId::FontAwesomeRegularFilePowerpoint,
                "doc" | "docm" | "docx" | "dot" => IconId::FontAwesomeRegularFileWord,
                "mp4" | "mov" | "wmv" | "avi" => IconId::FontAwesomeRegularFileVideo,
                "zip" | "zipx" | "rar" | "7z" | "gz" => IconId::FontAwesomeRegularFileZipper,
                "dat" | "pkl" | "bin" | "exe" => IconId::OcticonsFileBinary24,
                _ => IconId::FontAwesomeRegularFile,
            }
        }

        let Some(extension) = asset.path.as_path().extension() else {
            return IconId::FontAwesomeRegularFile;
        };

        let Some(extension) = extension.to_str() else {
            return IconId::FontAwesomeRegularFile;
        };

        get_icon_id(&extension.to_lowercase())
    }

    /// Gets the icon color to display for an [`Asset`](Asset).
    ///
    /// # Returns
    /// The `Color`.
    pub fn asset_icon_color(asset: &Asset) -> Color {
        let icon_id = asset_icon_id(asset);
        // TODO[l] Pull from stylesheet.
        let color = match icon_id {
            IconId::FontAwesomeRegularFileAudio => "#FFCC67",
            IconId::FontAwesomeRegularFileCode => "#B4DCE1",
            IconId::FontAwesomeRegularFileExcel => "#A8C764",
            IconId::FontAwesomeRegularFileImage => "#FFB800",
            IconId::FontAwesomeRegularFileLines => "#E0E2E8",
            IconId::FontAwesomeRegularFilePdf => "#E05C2B",
            IconId::FontAwesomeRegularFilePowerpoint => "#E97D55",
            IconId::FontAwesomeRegularFileWord => "#77B9CE",
            IconId::FontAwesomeRegularFileVideo => "#FFDC82",
            IconId::FontAwesomeRegularFileZipper => "#C8CCD4",
            IconId::OcticonsFileBinary24 => "#51A1C3",
            _ => "#F3F4F7",
        };
        format!("color: {}", color)
    }
}
