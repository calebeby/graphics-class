import { watch, run } from "watchlist";

const is_dev = process.argv.includes("--dev");

async function build() {
  console.log("running build..");
  await run(
    `$HOME/.cargo/bin/wasm-pack build --target web src/projects/proj-3 ${
      is_dev ? "--dev" : "--release"
    } --weak-refs`,
  );
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
