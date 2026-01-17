import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";

type Options = {
  input?: string;
  verbose: boolean;
  bin?: string;
};

function parseArgs(argv: string[]): Options {
  const options: Options = { verbose: false };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    switch (arg) {
      case "--input":
        options.input = argv[++i];
        break;
      case "--verbose":
        options.verbose = true;
        break;
      case "--bin":
        options.bin = argv[++i];
        break;
      case "--help":
      case "-h":
        printUsage();
        process.exit(0);
      default:
        console.error(`Unknown arg: ${arg}`);
        printUsage();
        process.exit(2);
    }
  }
  return options;
}

function resolveDefaultInput(): string {
  const candidates = [
    path.resolve(process.cwd(), "test_vectors/dleq.json"),
    path.resolve(__dirname, "../../../test_vectors/dleq.json"),
  ];
  for (const candidate of candidates) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }
  return candidates[0];
}

function buildCommand(options: Options): { cmd: string; args: string[] } {
  const input = options.input ?? resolveDefaultInput();
  if (options.bin) {
    const args = ["--input", input];
    if (options.verbose) {
      args.push("--verbose");
    }
    return { cmd: options.bin, args };
  }

  const args = ["run", "-p", "dleq_verifier", "--bin", "dleq-verify", "--", "--input", input];
  if (options.verbose) {
    args.push("--verbose");
  }
  return { cmd: "cargo", args };
}

function printUsage(): void {
  console.log("Usage: node dist/index.js [--input <path>] [--verbose] [--bin <path>]");
  console.log("Defaults to running the Rust verifier via cargo.");
}

function main(): void {
  const options = parseArgs(process.argv.slice(2));
  const { cmd, args } = buildCommand(options);
  const result = spawnSync(cmd, args, { stdio: "inherit" });
  process.exit(result.status ?? 1);
}

main();
