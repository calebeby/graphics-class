import { spawn } from "node:child_process";
import { homedir } from "node:os";
import { join } from "node:path";
import { watch } from "watchlist";

const is_dev = process.argv.includes("--dev");

async function build() {
  console.log("running build..");
  const wasm_pack_path = join(homedir(), ".cargo", "bin", "wasm-pack");
  console.log("wasm pack path:", wasm_pack_path);
  const [cmd, ...args] =
    `${wasm_pack_path} build --target web src/projects/proj-3 ${
      is_dev ? "--dev" : "--release"
    } --weak-refs`.split(" ");
  console.log("path is", process.env.PATH);
  const spawned = spawn(cmd, args, {
    stdio: "inherit",
    // eslint-disable-next-line @typescript-eslint/naming-convention
    env: { PATH: `${process.env.PATH} ${join(homedir(), ".cargo", "bin")}` },
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

if (is_dev) {
  let queued_build = false;

  const handle_change = () => {
    if (queued_build) return;
    setTimeout(() => {
      queued_build = false;
      build();
    }, 1);
    queued_build = true;
  };

  await watch(["src"], handle_change, {
    ignore: [
      "node_modules",
      "package.json",
      ".gitignore",
      /\.ts$/,
      /\.js$/,
      /\.wasm$/,
      "target",
      "Session.vim",
    ],
    clear: true,
    eager: true,
  });
} else {
  await build();
}
