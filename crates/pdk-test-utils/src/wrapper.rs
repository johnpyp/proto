use proto_core::Tool;
use proto_pdk_api::*;
use std::path::{Path, PathBuf};

pub struct WasmTestWrapper {
    pub tool: Tool,
}

impl WasmTestWrapper {
    pub fn from_virtual_path(&self, path: &Path) -> PathBuf {
        self.tool.plugin.from_virtual_path(path)
    }

    pub fn to_virtual_path(&self, path: &Path) -> PathBuf {
        self.tool.plugin.to_virtual_path(path)
    }

    pub fn create_shims(&self, input: CreateShimsInput) -> CreateShimsOutput {
        self.tool
            .plugin
            .call_func_with("create_shims", input)
            .unwrap()
    }

    pub fn detect_version(&self) -> DetectVersionOutput {
        self.tool.plugin.call_func("detect_version").unwrap()
    }

    pub fn download_prebuilt(&self, input: DownloadPrebuiltInput) -> DownloadPrebuiltOutput {
        self.tool
            .plugin
            .call_func_with("download_prebuilt", input)
            .unwrap()
    }

    pub fn load_versions(&self, input: LoadVersionsInput) -> LoadVersionsOutput {
        self.tool
            .plugin
            .call_func_with("load_versions", input)
            .unwrap()
    }

    pub fn locate_bins(&self, mut input: LocateBinsInput) -> LocateBinsOutput {
        if input.tool_dir.components().count() == 0 {
            input.tool_dir = self.tool.get_tool_dir();
        }

        input.home_dir = self.to_virtual_path(&input.home_dir);
        input.tool_dir = self.to_virtual_path(&input.tool_dir);

        let mut output: LocateBinsOutput = self
            .tool
            .plugin
            .call_func_with("locate_bins", input)
            .unwrap();

        if let Some(bin_path) = output.bin_path {
            output.bin_path = Some(self.from_virtual_path(&bin_path));
        }

        output
    }

    pub fn parse_version_file(&self, input: ParseVersionFileInput) -> ParseVersionFileOutput {
        self.tool
            .plugin
            .call_func_with("parse_version_file", input)
            .unwrap()
    }

    pub fn register_tool(&self, input: ToolMetadataInput) -> ToolMetadataOutput {
        self.tool
            .plugin
            .call_func_with("register_tool", input)
            .unwrap()
    }

    pub fn resolve_version(&self, input: ResolveVersionInput) -> ResolveVersionOutput {
        self.tool
            .plugin
            .call_func_with("resolve_version", input)
            .unwrap()
    }

    pub fn unpack_archive(&self, mut input: UnpackArchiveInput) {
        input.input_file = self.to_virtual_path(&input.input_file);
        input.output_dir = self.to_virtual_path(&input.output_dir);

        let _: EmptyInput = self
            .tool
            .plugin
            .call_func_with("unpack_archive", input)
            .unwrap();
    }

    pub fn verify_checksum(&self, mut input: VerifyChecksumInput) -> VerifyChecksumOutput {
        input.checksum_file = self.to_virtual_path(&input.checksum_file);
        input.download_file = self.to_virtual_path(&input.download_file);

        self.tool
            .plugin
            .call_func_with("verify_checksum", input)
            .unwrap()
    }
}
