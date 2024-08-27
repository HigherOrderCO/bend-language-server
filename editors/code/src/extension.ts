/* --------------------------------------------------------------------------------------------
 * Copyright (c) Higher Order Company (2024)
 * Based on original code by Microsoft Corporation.
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import {
  languages,
  workspace,
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
  commands,
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
  // let disposable = commands.registerCommand("helloworld.helloWorld", async uri => {
  //   // The code you place here will be executed every time your command is executed
  //   // Display a message box to the user
  //   let document = await workspace.openTextDocument(uri);
  //   await window.showTextDocument(document);
  //   // console.log(uri)
  //   window.activeTextEditor.document
  //   let editor = window.activeTextEditor;
  //   let range = new Range(1, 1, 1, 1)
  //   editor.selection = new Selection(range.start, range.end);
  // });
  // context.subscriptions.push(disposable);

  const logger: Logger = NullLogger; // FIXME
  const traceOutputChannel = window.createOutputChannel("Bend Language Server trace");
  const command = process.env.SERVER_PATH || pipe(
    await findLanguageServer(context, logger),
    E.match(
      error => { throw new Error(`Could not find language server: ${error}`); },
      languageServer => languageServer
    )
  );

  // We have to check if `bend-language-server` is installed, and if it's not, try to install it.

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
