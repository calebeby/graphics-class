import { spawn } from "node:child_process";
import { homedir } from "node:os";
import { join } from "node:path";
import { watch } from "watchlist";
import * as fs from "node:fs/promises";

const args = process.argv.slice(2).filter((argv) => !argv.startsWith("--"));
const is_dev = process.argv.includes("--dev");
const is_watch = process.argv.includes("--watch");

const projects =
  args.length > 0
    ? args
    : (await fs.readdir("src/projects")).filter(
        (p) => p !== "proj-1" && p !== "proj-2" && !p.startsWith("--"),
      );

async function build(project) {
  console.log(`\nRunning build (${project})...`);
  const [cmd, ...args] =
    `wasm-pack build --target web src/projects/${project} ${
      is_dev ? "--dev" : "--release"
    }`.split(" ");
  const spawned = spawn(cmd, args, {
    stdio: "inherit",
    shell: "/usr/bin/bash",
    env: {
      // eslint-disable-next-line @typescript-eslint/naming-convention
      PATH: `${process.env.PATH}:${join(homedir(), ".cargo", "bin")}`,
      // eslint-disable-next-line @typescript-eslint/naming-convention
      CARGO_TERM_COLOR: "always",
    },
  });
  await new Promise((resolve, reject) => {
    spawned.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        const msg = `command failed with exit code ${code}`;
        console.log(msg);
        if (is_dev) {
          resolve();
        } else {
          reject(msg);
        }
      }
    });
  });
}

for (const project of projects) {
  await build(project).catch((error) => console.error(error));
}
if (is_watch) {
  for (const project of projects) {
    let queued_build = false;

    const handle_change = () => {
      if (queued_build) return;
      setTimeout(() => {
        build(project)
          .catch((error) => console.error(error))
          .then(() => {
            queued_build = false;
          });
      }, 1);
      queued_build = true;
    };

    await watch([`src/projects/${project}`], handle_change, {
      ignore: [
        /package\.json$/,
        /\.gitignore/,
        /\.ts$/,
        /\.tsx$/,
        /\.js$/,
        /\.css$/,
        /\.glsl$/,
        /\.png$/,
        /\.jpg$/,
        /\.jpeg$/,
        /\.webp$/,
        /\.wasm$/,
        /Session\.vim/,
        /~$/,
        /^\d*$/,
      ],
      clear: false,
    });
  }
}
