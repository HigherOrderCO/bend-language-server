/* --------------------------------------------------------------------------------------------
 * Copyright (c) Higher Order Company (2024)
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 * 
 * This entire file is based on the Haskell VSCode extension (https://github.com/haskell/vscode-haskell),
 * which is also licensed under the MIT license.
 * ------------------------------------------------------------------------------------------ */

import { ConfigurationTarget, ExtensionContext, ProgressLocation, window, workspace, WorkspaceFolder } from "vscode";
import { Logger } from "vscode-languageclient";
import { match } from 'ts-pattern';
import { execPath } from "process";
import * as child_process from 'child_process';
import * as fs from "fs";
import * as os from "os";
import which from "which";
import dedent from "dedent-js";

// This module deals with pulling and installing the language server binaries into the system.

// The language server can be installed automatically by the extension, be used from the PATH environment
// variable, or configured from a specific location by the user
type ManageLanguageServer = "automatic" | "PATH";
let manageLS = workspace
  .getConfiguration("bend")
  .get("manageLanguageServer") as ManageLanguageServer;

// On Windows, the executable needs to be stored somewhere with an .exe extension
const executableExtension = process.platform === "win32" ? ".exe" : "";

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
): Promise<[string, string | undefined]> {
  logger.info("Looking for the language server");

  let serverExecutablePath = workspace
    .getConfiguration("bend")
    .get("serverExecutablePath") as string;
  if (serverExecutablePath) {
    // Get the language server from the configured location
    const executable = await findServerExecutable(logger, folder);
    return [executable, undefined];
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

  if (manageLS == "PATH") {
    const executable = await findServerExecutableinPATH(context, logger);
    return [executable, undefined];
  }

  // If we're here, automatically install the latest `bend-language-server` binary.



  throw "TODO";
}

async function findServerExecutable(logger: Logger, folder?: WorkspaceFolder): Promise<string> {
  let executablePath = workspace.getConfiguration("bend").get("serverExecutablePath") as string;
  logger.info(`Trying to find the Bend Language Server binary at ${executablePath}`);

  executablePath = resolvePathPlaceHolders(executablePath, folder);
  logger.info(`Resolved path variables: ${executablePath}`);

  if (executableExists(executablePath)) {
    return executablePath;
  } else {
    throw new Error(dedent`
      Could not find the Bend Language Server at ${execPath}.
      Consider changing settings for "bend.manageLanguageServer" or "bend.serverExecutablePath".
      `);
  }
}

async function findServerExecutableinPATH(_context: ExtensionContext, logger: Logger): Promise<string> {
  const executable = "bend-language-server";
  if (executableExists(executable)) {
    logger.info(`Found server executable in $PATH: ${executable}`);
    return executable;
  }
  throw new Error('Could not find the bend-language-server binary in PATH.');
}

export function resolvePathPlaceHolders(path: string, folder?: WorkspaceFolder) {
  path = path.replace('${HOME}', os.homedir).replace('${home}', os.homedir).replace(/^~/, os.homedir);
  if (folder) {
    path = path.replace('${workspaceFolder}', folder.uri.path).replace('${workspaceRoot}', folder.uri.path);
  }
  return path;
}

export function resolvePATHPlaceHolders(path: string) {
  return path
    .replace('${HOME}', os.homedir)
    .replace('${home}', os.homedir)
    .replace('$PATH', process.env.PATH ?? '$PATH')
    .replace('${PATH}', process.env.PATH ?? '${PATH}');
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
    ? serverEnv.PATH.split(pathSep).map((p) => resolvePATHPlaceHolders(p))
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

async function findCargo(_context: ExtensionContext, logger: Logger, folder?: WorkspaceFolder): Promise<string> {
  logger.info("Looking for Cargo...");

  let cargoExecutable = workspace.getConfiguration("bend").get("cargoExecutablePath") as string;
  if (cargoExecutable) {
    logger.info(`Looking for Cargo in ${cargoExecutable}...`);
    cargoExecutable = resolvePathPlaceHolders(cargoExecutable, folder);
    logger.info(`Translated the path to ${cargoExecutable}`);
    if (executableExists(cargoExecutable)) {
      return cargoExecutable;
    } else {
      throw new Error(`Could not find Cargo at ${cargoExecutable}`);
    }
  }

  if (executableExists("cargo")) {
    // Found it in PATH
    return "cargo"
  }

  // We'll try to find the `cargo by looking around
  logger.info("Probing for Cargo...");
  cargoExecutable = match(process.platform)
    .with("win32", () => {
      // use homedir
      return ""
    });

}

async function callAsync(
  binary: string,
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
        const command: string = binary + ' ' + args.join(' ');
        logger.info(`Executing '${command}' in cwd '${dir ? dir : process.cwd()}'`);
        token.onCancellationRequested(() => {
          logger.warn(`User canceled the execution of '${command}'`);
        });
        // Need to set the encoding to 'utf8' in order to get back a string
        // We execute the command in a shell for windows, to allow use .cmd or .bat scripts
        const childProcess = child_process
          .execFile(
            process.platform === 'win32' ? `"${binary}"` : binary,
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
