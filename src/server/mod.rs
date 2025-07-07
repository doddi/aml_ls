pub mod capabilities;

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::fmt::time::ChronoUtc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::core::document_manager::DocumentManager;
use crate::core::goto_manager::GotoManager;
use crate::features::diagnostics::DiagnosticProvider;
use crate::features::hover::{HoverContext, HoverProvider};

#[derive(Debug)]
pub struct Backend {
    client: Client,
    document_manager: DocumentManager,
    hover_provider: HoverProvider,
    goto_provider: GotoManager,
    // diagnostic_provider: DiagnosticProvider,
}


#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, initialize_parameters: InitializeParams) -> Result<InitializeResult> {
        if let Some(uri) = initialize_parameters.root_uri {
            self.goto_provider.gather_aml_files_for_project(Path::new(uri.path()))
                .expect("failed to load aml files");
        }

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "AML Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: capabilities::server_capabilities(),
        })
    }

    async fn initialized(&self, _: InitializedParams) {}

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.document_manager.did_open(params).await;
        // self.publish_diagnostics(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.document_manager.did_change(params).await;
        // self.publish_diagnostics(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.document_manager.did_close(params).await;
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>>
    {
        match self.goto_provider.get_url_for_token("basic") {
            None => Ok(None),
            Some(location) => {
                Ok(
                    Some(
                        GotoDefinitionResponse::Scalar(
                            Location::new(
                                location,
                                Range::new(Position::new(0, 0), Position::new(0, 0))
                            )
                        )
                    )
                )
            }
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.hover_provider
            .hover(HoverContext {
                document_manager: &self.document_manager,
                params,
            })
            .await
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        tracing::info!("did_change_workspace_folders: {:?}", params);
    }

    async fn did_create_files(&self, params: CreateFilesParams) {
        tracing::info!("did_create_files: {:?}", params);
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
       tracing::info!("did_rename_files: {:?}", params);
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        tracing::info!("did_delete_files: {:?}", params);
    }
}

impl Backend {
    // async fn publish_diagnostics(&self, uri: &Url) {
    //     let diagnostics = self
    //         .diagnostic_provider
    //         .get_diagnostics(&self.document_manager, uri)
    //         .await;
    //     self.client
    //         .publish_diagnostics(uri.clone(), diagnostics, None)
    //         .await;
    // }
}

fn init_tracing() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let log_dir = PathBuf::from("/tmp/aml_ls");
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::never(&log_dir, "aml_ls.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_timer(ChronoUtc::rfc_3339())
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_target(false),
        )
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")))
        .init();

    tracing::info!("AML Language Server starting up");
    // Leak the guard to keep the writer alive
    std::mem::forget(_guard);
    Ok(())
}

pub async fn start() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    init_tracing().ok();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        document_manager: DocumentManager::new(),
        hover_provider: HoverProvider::new(),
        goto_provider: GotoManager::default(),
        // diagnostic_provider: DiagnosticProvider::new(),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
