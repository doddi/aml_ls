use tower_lsp::lsp_types::*;

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        // text_document_sync: Some(TextDocumentSyncCapability::Kind(
        //     TextDocumentSyncKind::INCREMENTAL,
        // )),
        // hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                will_create: None,
                did_create: Some(FileOperationRegistrationOptions {
                    filters: vec![FileOperationFilter {
                        scheme: None,
                        pattern: FileOperationPattern {
                            glob: "**/*".to_string(),
                            matches: Some(FileOperationPatternKind::File),
                            options: None,
                        },
                    }],
                }),
                will_rename: None,
                did_rename: Some(FileOperationRegistrationOptions {
                    filters: vec![FileOperationFilter {
                        scheme: None,
                        pattern: FileOperationPattern {
                            glob: "**/*.aml".to_string(),
                            matches: Some(FileOperationPatternKind::File),
                            options: None,
                        },
                    }],
                }),
                will_delete: None,
                did_delete: Some(FileOperationRegistrationOptions {
                    filters: vec![FileOperationFilter {
                        scheme: None,
                        pattern: FileOperationPattern {
                            glob: "**/*.aml".to_string(),
                            matches: Some(FileOperationPatternKind::File),
                            options: None,
                        },
                    }],
                }),
            }),
        }),
        // diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
        //     identifier: Some("aml-ls".to_string()),
        //     inter_file_dependencies: false,
        //     workspace_diagnostics: false,
        //     work_done_progress_options: WorkDoneProgressOptions::default(),
        // })),
        ..ServerCapabilities::default()
    }
}
