use futures::future::try_join_all;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tracing::debug;

use crate::parser::query_rokit_toml_dependencies;
use crate::parser::SimpleDependency;
use crate::server::*;
use crate::util::*;

use super::*;

mod completion;
mod constants;
mod diagnostics;
mod hover;

use completion::*;
use diagnostics::*;
use hover::*;

#[derive(Debug, Clone)]
pub struct Rokit {
    _client: Client,
    clients: Clients,
    documents: Documents,
}

impl Rokit {
    pub(super) fn new(client: Client, clients: Clients, documents: Documents) -> Self {
        Self {
            _client: client,
            clients,
            documents,
        }
    }

    fn get_document(&self, uri: &Url) -> Option<Document> {
        if uri.file_name().as_deref().is_some_and(|f| {
            f.eq_ignore_ascii_case("rokit.toml") || f.eq_ignore_ascii_case("aftman.toml")
        }) {
            self.documents.get(uri).map(|r| r.clone())
        } else {
            None
        }
    }
}

#[tower_lsp::async_trait]
impl Tool for Rokit {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some(doc) = self.get_document(&uri) else {
            return Ok(None);
        };

        // Find the dependency that is hovered over
        let dependencies = query_rokit_toml_dependencies(doc.inner());
        let Some(found) = SimpleDependency::find_at_pos(&dependencies, pos) else {
            return Ok(None);
        };

        // Fetch some extra info and return the hover
        debug!("Hovering: {found:?}");
        get_rokit_hover(&self.clients, &doc, found).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<CompletionResponse> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let Some(doc) = self.get_document(&uri) else {
            return Ok(CompletionResponse::Array(Vec::new()));
        };

        // Find the dependency that is being completed
        let dependencies = query_rokit_toml_dependencies(doc.inner());
        let Some(found) = SimpleDependency::find_at_pos(&dependencies, pos) else {
            return Ok(CompletionResponse::Array(Vec::new()));
        };

        // Check what we're completing - author, name, or version
        let parsed = found.parsed_spec();
        if parsed.version.as_ref().is_some_and(|v| v.contains(pos)) {
            debug!("Completing version: {found:?}");
            return get_rokit_completions_spec_version(&self.clients, &doc, found).await;
        } else if parsed.name.is_some_and(|n| n.contains(pos)) {
            debug!("Completing name: {found:?}");
            return get_rokit_completions_spec_name(&self.clients, &doc, found).await;
        } else if parsed.author.contains(pos)
            || (parsed.author.unquoted().is_empty() && found.spec.contains(pos))
        {
            debug!("Completing author: {found:?}");
            return get_rokit_completions_spec_author(&self.clients, &doc, found).await;
        }

        Ok(CompletionResponse::Array(Vec::new()))
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        let uri = params.text_document.uri;
        let Some(doc) = self.get_document(&uri) else {
            return Ok(Vec::new());
        };

        // Find all dependencies
        let dependencies = query_rokit_toml_dependencies(doc.inner());
        if dependencies.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch all diagnostics concurrently
        debug!("Fetching rokit diagnostics for dependencies");
        let results = try_join_all(
            dependencies
                .iter()
                .map(|tool| get_rokit_diagnostics(&self.clients, &doc, tool)),
        )
        .await?;

        Ok(results.into_iter().flatten().collect())
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Vec<CodeActionOrCommand>> {
        let mut actions = Vec::new();
        for diag in params.context.diagnostics {
            if let Some(Ok(action)) = diag
                .data
                .as_ref()
                .map(ResolveContext::<CodeActionMetadata>::try_from)
            {
                actions.push(action.into_inner().into_code_action(diag.clone()))
            }
        }
        Ok(actions)
    }
}
