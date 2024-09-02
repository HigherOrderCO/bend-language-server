/* --------------------------------------------------------------------------------------------
 * Copyright (c) Higher Order Company (2024)
 * Based on original code by Microsoft Corporation under the MIT License.
 * Licensed under the MIT License. See LICENSE in the project root for information.
 * ------------------------------------------------------------------------------------------ */

import {
  languages,
  workspace,
  commands,
  EventEmitter,
  ExtensionContext,
  window,
  InlayHintsProvider,
  TextDocument,
  CancellationToken,
  Range,
  InlayHint,
  TextDocumentChangeEvent,
  ProviderResult,
  WorkspaceEdit,
  TextEdit,
  Selection,
  Uri,
} from "vscode";

import {
  Disposable,
  Executable,
  LanguageClient,
  LanguageClientOptions,
  Logger,
  NullLogger,
  ServerOptions,
} from "vscode-languageclient/node";

import { pipe } from "fp-ts/lib/function";
import * as E from "fp-ts/Either";

import { findLanguageServer } from "./binaries";

let client: LanguageClient;

// ===================
// Main extension code

export async function activate(context: ExtensionContext) {
  const logger: Logger = NullLogger; // FIXME
  const traceOutputChannel = window.createOutputChannel("Bend Language Server trace");
  traceOutputChannel.appendLine(`Local file storage: ${context.globalStorageUri.fsPath}`);

  // Register editor commands
  const restartCommand = commands.registerCommand("bend.commands.restartServer", async () => {
    if (client.isRunning()) {
      client.info("Stopping the language server.");
      await client.stop();
    }
    client.info("Starting the language server.");
    await client.start();
  });
  context.subscriptions.push(restartCommand);

  const stopCommand = commands.registerCommand("bend.commands.stopServer", async () => {
    client.info("Stopping the language server.");
    await client.stop();
  });
  context.subscriptions.push(stopCommand);

  // We have to check if `bend-language-server` is installed, and if it's not, try to install it.
  const command = process.env.BEND_LS_PATH || pipe(
    await findLanguageServer(context, logger),
    E.match(
      error => { throw new Error(error); },
      languageServer => languageServer
    )
  );

  const run: Executable = {
    command,
    options: { env: { ...process.env, RUST_LOG: "info" } }
  };

  const debug: Executable = {
    command,
    options: { env: { ...process.env, RUST_LOG: "debug" } }
  };

  const serverOptions: ServerOptions = { run, debug, };

  let clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [{ scheme: "file", language: "bend" }],
    synchronize: {
      // Notify the server about file changes to '.clientrc files contained in the workspace
      // (we don't care about this for now, it was part of the boilerplate)
      fileEvents: workspace.createFileSystemWatcher("**/.clientrc"),
    },
    traceOutputChannel,
  };

  // Create the language client and start the client.
  client = new LanguageClient("bend-language-server", "Bend Language Server", serverOptions, clientOptions);
  // activateInlayHints(context);
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}

// ================
// Future additions

export function activateInlayHints(ctx: ExtensionContext) {
  const maybeUpdater = {
    hintsProvider: null as Disposable | null,
    updateHintsEventEmitter: new EventEmitter<void>(),

    async onConfigChange() {
      this.dispose();

      const event = this.updateHintsEventEmitter.event;
      this.hintsProvider = languages.registerInlayHintsProvider(
        { scheme: "file", language: "bend" },
        new (class implements InlayHintsProvider {
          onDidChangeInlayHints = event;
          resolveInlayHint(hint: InlayHint, token: CancellationToken): ProviderResult<InlayHint> {
            const ret = {
              label: hint.label,
              ...hint,
            };
            return ret;
          }
          async provideInlayHints(
            document: TextDocument,
            range: Range,
            token: CancellationToken
          ): Promise<InlayHint[]> {
            const hints = (await client
              .sendRequest("custom/inlay_hint", { path: document.uri.toString() })
              .catch(err => null)) as [number, number, string][];
            if (hints == null) {
              return [];
            } else {
              return hints.map(item => {
                const [start, end, label] = item;
                let startPosition = document.positionAt(start);
                let endPosition = document.positionAt(end);
                return {
                  position: endPosition,
                  paddingLeft: true,
                  label: [
                    {
                      value: `${label}`,
                      location: {
                        uri: document.uri,
                        range: new Range(1, 0, 1, 0)
                      },
                      command: {
                        title: "hello world",
                        command: "helloworld.helloWorld",
                        arguments: [document.uri],
                      },
                    },
                  ],
                };
              });
            }
          }
        })()
      );
    },

    onDidChangeTextDocument({ contentChanges, document }: TextDocumentChangeEvent) {
      // debugger
      // this.updateHintsEventEmitter.fire();
    },

    dispose() {
      this.hintsProvider?.dispose();
      this.hintsProvider = null;
      this.updateHintsEventEmitter.dispose();
    },
  };

  workspace.onDidChangeConfiguration(maybeUpdater.onConfigChange, maybeUpdater, ctx.subscriptions);
  workspace.onDidChangeTextDocument(maybeUpdater.onDidChangeTextDocument, maybeUpdater, ctx.subscriptions);

  maybeUpdater.onConfigChange().catch(console.error);
}
