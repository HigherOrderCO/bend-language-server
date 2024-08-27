/* --------------------------------------------------------------------------------------------
 * Copyright (c) Higher Order Company (2024)
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 * 
 * This entire file is based on the Haskell VSCode extension (https://github.com/haskell/vscode-haskell),
 * which is also licensed under the MIT license.
 * ------------------------------------------------------------------------------------------ */

// This module deals with pulling and installing the language server binaries into the system.

import { ConfigurationTarget, ExtensionContext, ProgressLocation, window, workspace, WorkspaceFolder } from "vscode";
import { Logger } from "vscode-languageclient";
import { match } from 'ts-pattern';
import { execPath } from "process";
import * as child_process from 'child_process';
import * as fs from "fs";
import * as os from "os";
import which from "which";
import dedent from "dedent-js";
import path from "path";
import * as TE from "fp-ts/lib/TaskEither";
import * as E from "fp-ts/lib/Either";
import { pipe } from "fp-ts/lib/function";

const BEND_LS_EXE = "bend-language-server";
// On Windows, the executable needs to be stored with a .exe extension
const EXT = process.platform === "win32" ? ".exe" : "";

// The language server can be installed automatically by the extension, be used from the PATH environment
// variable, or configured from a specific location by the user
type ManageLanguageServer = "automatic" | "PATH";
let manageLS = workspace
  .getConfiguration("bend")
  .get("manageLanguageServer") as ManageLanguageServer;

type ProcessCallback = (
  error: child_process.ExecFileException | null,
  stdout: string,
  stderr: string,
  resolve: (value: string | PromiseLike<string>) => void,
  reject: (reason?: Error | string) => void
) => void;

export interface IEnvVars {
  [key: string]: string;
}

export async function findLanguageServer(
  context: ExtensionContext,
  logger: Logger,
  workingDir: string,
  folder?: WorkspaceFolder
): Promise<E.Either<string, string>> {
  logger.info("Looking for the language server");

  let serverExecutablePath = workspace
    .getConfiguration("bend")
    .get("serverExecutablePath") as string;
  if (serverExecutablePath) {
    // Get the language server from the configured location
    return await findConfiguredServerExecutable(logger, folder);
  }

  const storagePath = await getStoragePath(context);

  if (!fs.existsSync(storagePath)) {
    fs.mkdirSync(storagePath);
  }

  // First initialization, ask the user how they want to manage the language server
  if (!context.globalState.get("pluginInitialized") as boolean | null) {
    const message =
      "How would you like to manage the Bend Language Server binary?";

    const popup = window.showInformationMessage(
      message,
      { modal: true },
      "Automatically (compile via Cargo)",
      "Manually via PATH environment variable"
    );

    const decision = (await popup) || null;
    match(decision)
      .with("Automatically (compile via Cargo)", () => manageLS = "automatic")
      .with("Manually via PATH environment variable", () => manageLS = "PATH")
      .otherwise(() => {
        window.showWarningMessage("Choosing to install the Bend Language Server automatically via Cargo.")
        manageLS = "automatic";
      });

    workspace.getConfiguration("bend").update("manageLanguageServer", manageLS, ConfigurationTarget.Global);
    context.globalState.update("pluginInitialized", true);
  }

  // User wants us to grab `bend-language-server` from the PATH environment variable
  if (manageLS == "PATH") {
    return await findExecutableInPATH(BEND_LS_EXE, context, logger);
  }

  // Automatically install the latest `bend-language-server` binary.
  if (manageLS == "automatic") {
    // -> check if installed locally
    //   -> yes: check if latest version
    //     -> yes: ask to update
    //       -> yes: update and run
    //       -> no: just run
    //     -> no: just run
    //   -> no: install and run

    let languageServer = await managedServerExecutableLocation(context);
    if (!executableExists(languageServer)) {
      // Install the `bend-language-server` binary.
      return pipe(
        await getLanguageServerLatestVersion(context, logger),
        E.chain(version => installLanguageServer(version, context, logger))
      )
    } else {
      // Check if latest version is installed.
      const newVersion = await getLanguageServerLatestVersion(context, logger);

      const currentVersion =
        (await callAsync(
          languageServer,
          ["--version"],
          logger,
          undefined,
          "Checking current installed version...",
          true
        )).trim();

      return await pipe(newVersion,
        E.match(
          async _error => {
            // We couldn't find the newest version for some reason...
            // Keep this one.
            return E.right(languageServer);
          },
          async version => {
            if (version !== currentVersion) {
              return await askToUpdateLanguageServer(languageServer, version, context, logger);
            } else {
              return E.right(languageServer);
            }
          }
        )
      )
    }
  }

  return E.left("Invalid configuration for managing the Bend Language Server.");
}

async function managedServerExecutableLocation(context: ExtensionContext): Promise<string> {
  return path.join(await getStoragePath(context), "bin", `${BEND_LS_EXE}${EXT}`);
}

async function getLanguageServerLatestVersion(context: ExtensionContext, logger: Logger): Promise<E.Either<string, string>> {
  return pipe(
    await callCargo(
      ["search", BEND_LS_EXE],
      context,
      logger,
      "Searching for bend-language-server...",
      true
    ),
    E.map(
      output => {
        let version = /".*"/.exec(output)[0];
        version = version.substring(1, version.length - 1); // quotes
        return version;
      }
    ));
}

async function askToUpdateLanguageServer(languageServer: string, version: string, context: ExtensionContext, logger: Logger): Promise<E.Either<string, string>> {
  const decision = await window.showInformationMessage(
    `There is a new version of the Bend Language Server (${version}). Would you like to install it?`,
    "Yes",
    "No"
  );

  if (decision == "No") {
    return E.right(languageServer);
  }

  // Update the language server
  logger.info(`Installing bend-language-server ${version}`);

  return await installLanguageServer(version, context, logger);
}

async function installLanguageServer(version: string, context: ExtensionContext, logger: Logger): Promise<E.Either<string, string>> {
  return pipe(
    await callCargo(
      [BEND_LS_EXE, "--version", version, "--root", await getStoragePath(context)],
      context,
      logger,
      "Installing the Bend Language Server...",
      true
    ),
    E.chain(
      async _output => E.right(await managedServerExecutableLocation(context))
    )
  );
}

async function findConfiguredServerExecutable(logger: Logger, folder?: WorkspaceFolder): Promise<E.Either<string, string>> {
  let executablePath = workspace.getConfiguration("bend").get("serverExecutablePath") as string;
  logger.info(`Trying to find the Bend Language Server binary at ${executablePath}`);

  executablePath = resolvePlaceHolders(executablePath, folder);
  logger.info(`Resolved path variables: ${executablePath}`);

  if (executableExists(executablePath)) {
    return E.right(executablePath);
  } else {
    return E.left(dedent`
      Could not find the Bend Language Server at ${execPath}.
      Consider changing settings for "bend.manageLanguageServer" or "bend.serverExecutablePath".
      `);
  }
}

async function findExecutableInPATH(executable: string, _context: ExtensionContext, logger: Logger): Promise<E.Either<string, string>> {
  if (executableExists(executable)) {
    return E.right(executable);
  } else {
    return E.left(`Could not find ${executable} in PATH.`)
  }
}

export function resolvePlaceHolders(path: string, folder?: WorkspaceFolder) {
  path = path
    .replace('${HOME}', os.homedir)
    .replace('${home}', os.homedir)
    .replace(/^~/, os.homedir)
    .replace('$PATH', process.env.PATH ?? '$PATH')
    .replace('${PATH}', process.env.PATH ?? '${PATH}')
    .replace('${CARGO_HOME}', process.env.CARGO_HOME);

  if (folder) {
    path = path
      .replace('${workspaceFolder}', folder.uri.path)
      .replace('${workspaceRoot}', folder.uri.path);
  }
  return path;
}

export function executableExists(exe: string): boolean {
  const isWindows = process.platform === 'win32';
  let newEnv: IEnvVars = resolveServerEnvironmentPATH(
    workspace.getConfiguration('bend').get('serverEnvironment') || {}
  );
  newEnv = { ...(process.env as IEnvVars), ...newEnv };
  const cmd: string = isWindows ? 'where' : 'which';
  const out = child_process.spawnSync(cmd, [exe], { env: newEnv });
  return out.status === 0 || (which.sync(exe, { nothrow: true, path: newEnv.PATH }) ?? '') !== '';
}

export function resolveServerEnvironmentPATH(serverEnv: IEnvVars): IEnvVars {
  const pathSep = process.platform === 'win32' ? ';' : ':';
  const path: string[] | null = serverEnv.PATH
    ? serverEnv.PATH.split(pathSep).map((p) => resolvePlaceHolders(p))
    : null;
  return {
    ...serverEnv,
    ...(path ? { PATH: path.join(pathSep) } : {}),
  };
}

async function getStoragePath(
  context: ExtensionContext
): Promise<string> {
  return context.globalStorageUri.fsPath;
}

async function findCargo(_context: ExtensionContext, logger: Logger, folder?: WorkspaceFolder): Promise<E.Either<string, string>> {
  logger.info("Looking for Cargo...");

  let cargoExecutable = workspace.getConfiguration("bend").get("cargoExecutablePath") as string;
  if (cargoExecutable) {
    logger.info(`Looking for Cargo in ${cargoExecutable}...`);
    cargoExecutable = resolvePlaceHolders(cargoExecutable, folder);
    logger.info(`Translated the path to ${cargoExecutable}`);
    if (executableExists(cargoExecutable)) {
      return E.right(cargoExecutable);
    } else {
      return E.left(`Could not find Cargo at ${cargoExecutable}`);
    }
  }

  if (executableExists("cargo")) {
    // Found it in PATH
    return E.right("cargo");
  }

  // We'll try to find the cargo binary by looking around
  logger.info("Probing for Cargo...");

  cargoExecutable = path.join(process.env.CARGO_HOME, ".cargo", "bin", "cargo");
  if (executableExists(cargoExecutable)) return E.right(cargoExecutable);

  cargoExecutable = path.join(os.homedir(), ".cargo", "bin", "cargo");
  if (executableExists(cargoExecutable)) return E.right(cargoExecutable);

  return E.left("Could not find the Cargo binary.");
}

async function callCargo(
  args: string[],
  context: ExtensionContext,
  logger: Logger,
  title?: string,
  cancellable?: boolean,
  callback?: ProcessCallback
): Promise<E.Either<string, string>> {
  if (manageLS !== "automatic") {
    return E.left("Tried to call Cargo when bend.manageLanguageServer is not set to automatic.");
  }

  return pipe(
    await findCargo(context, logger),
    E.match(
      async (error) => E.left(error),
      async (cargo) => E.right(
        await callAsync(
          cargo,
          args,
          logger,
          undefined,
          title,
          cancellable,
          {},
          callback
        ))
    )
  )
}

async function callAsync(
  executablePath: string,
  args: string[],
  logger: Logger,
  dir?: string,
  title?: string,
  cancellable?: boolean,
  envAdd?: IEnvVars,
  callback?: ProcessCallback
): Promise<string> {
  let newEnv: IEnvVars = resolveServerEnvironmentPATH(
    workspace.getConfiguration('bend').get('serverEnvironment') || {}
  );
  newEnv = { ...(process.env as IEnvVars), ...newEnv, ...(envAdd || {}) };
  return window.withProgress(
    {
      location: ProgressLocation.Notification,
      title,
      cancellable,
    },
    async (_, token) => {
      return new Promise<string>((resolve, reject) => {
        const command: string = executablePath + ' ' + args.join(' ');
        logger.info(`Executing '${command}' in cwd '${dir ? dir : process.cwd()}'`);
        token.onCancellationRequested(() => {
          logger.warn(`User canceled the execution of '${command}'`);
        });
        // Need to set the encoding to 'utf8' in order to get back a string
        // We execute the command in a shell for windows, to allow use .cmd or .bat scripts
        const childProcess = child_process
          .execFile(
            process.platform === 'win32' ? `"${executablePath}"` : executablePath,
            args,
            { encoding: 'utf8', cwd: dir, shell: process.platform === 'win32', env: newEnv },
            (err, stdout, stderr) => {
              if (err) {
                logger.error(`Error executing '${command}' with error code ${err.code}`);
                logger.error(`stderr: ${stderr}`);
                if (stdout) {
                  logger.error(`stdout: ${stdout}`);
                }
              }
              if (callback) {
                callback(err, stdout, stderr, resolve, reject);
              } else {
                if (err) {
                  reject(
                    Error(`\`${command}\` exited with exit code ${err.code}.`)
                  );
                } else {
                  resolve(stdout?.trim());
                }
              }
            }
          )
          .on('exit', (code, signal) => {
            const msg =
              `Execution of '${command}' terminated with code ${code}` + (signal ? `and signal ${signal}` : '');
            logger.log(msg);
          })
          .on('error', (err) => {
            if (err) {
              logger.error(`Error executing '${command}': name = ${err.name}, message = ${err.message}`);
              reject(err);
            }
          });
        token.onCancellationRequested(() => childProcess.kill());
      });
    }
  );
}
