// Regenerates `crates/rsvelte_language_server/src/html_data/web.rs` from the
// HTML data the official language server itself reads.
//
//   node scripts/dev/generate-html-data.mjs [--package-root <dir>]
//
// The version is not a constant here: it is read out of language-tools'
// `pnpm-lock.yaml`, and the resolved package has to agree with it. The
// SHA-256 of every file read goes into the generated header, so the identity
// of the input is asserted from its content rather than from where it lived.
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const PACKAGE = "vscode-html-languageservice";
const ROOT = path.resolve(fileURLToPath(import.meta.url), "../../..");
const LOCKFILE = path.join(ROOT, "submodules/language-tools/pnpm-lock.yaml");
const OUTPUT = path.join(
  ROOT,
  "crates/rsvelte_language_server/src/html_data/web.rs",
);
const ORACLE = path.join(
  ROOT,
  "crates/rsvelte_language_server/tests/data/html-documentation.json",
);
const DATA_FILE = "lib/umd/languageFacts/data/webCustomData.js";
// `package.json` `main` is the umd build, so umd is what the official server
// loads; the esm copy of the same data hashes differently.
const PROVIDER_FILE = "lib/umd/languageFacts/dataProvider.js";

function lockedVersion() {
  const lock = fs.readFileSync(LOCKFILE, "utf8");
  const versions = new Set(
    [...lock.matchAll(new RegExp(`^  ${PACKAGE}@([^:\\s]+):`, "gm"))].map(
      (match) => match[1],
    ),
  );
  if (versions.size !== 1) {
    throw new Error(
      `${LOCKFILE} pins ${versions.size} versions of ${PACKAGE}: ${[...versions].join(", ")}`,
    );
  }
  return [...versions][0];
}

function packageRoot(override, version) {
  const candidate =
    override ??
    path.join(
      ROOT,
      "submodules/language-tools/node_modules/.pnpm",
      `${PACKAGE}@${version}/node_modules/${PACKAGE}`,
    );
  const manifest = path.join(candidate, "package.json");
  if (!fs.existsSync(manifest)) {
    throw new Error(
      `${PACKAGE} is not installed at ${candidate}. Run \`pnpm install\` in submodules/language-tools, or pass --package-root.`,
    );
  }
  const { name, version: resolved } = JSON.parse(
    fs.readFileSync(manifest, "utf8"),
  );
  if (name !== PACKAGE || resolved !== version) {
    throw new Error(
      `${candidate} is ${name}@${resolved}, but ${LOCKFILE} pins ${PACKAGE}@${version}`,
    );
  }
  return candidate;
}

const digest = (file) =>
  crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");

const string = (value) => JSON.stringify(value);
const option = (value) => (value === undefined ? "None" : `Some(${string(value)})`);

/// `normalizeMarkupContent` (`utils/markup.js`) reads both spellings of a
/// description as markdown, so only the text survives the round trip.
const description = (item) =>
  option(
    typeof item.description === "string"
      ? item.description
      : item.description?.value,
  );

const slice = (items, render) =>
  items.length === 0 ? "&[]" : `&[${items.map(render).join(",")}]`;

const references = (item) =>
  slice(
    item.references ?? [],
    (reference) =>
      `Reference{name:${string(reference.name)},url:${string(reference.url)}}`,
  );

const browsers = (item) =>
  slice(item.browsers ?? [], (browser) => string(browser));

function status(item) {
  if (!item.status) {
    return "None";
  }
  const baseline = {
    high: "Baseline::High",
    low: "Baseline::Low",
    false: "Baseline::Limited",
  }[String(item.status.baseline)];
  if (!baseline) {
    throw new Error(`unknown baseline ${JSON.stringify(item.status.baseline)}`);
  }
  return `Some(Status{baseline:${baseline},low_date:${option(item.status.baseline_low_date)},high_date:${option(item.status.baseline_high_date)}})`;
}

const attribute = (item) =>
  `Attribute{name:${string(item.name)},description:${description(item)},value_set:${option(item.valueSet)},references:${references(item)},browsers:${browsers(item)},status:${status(item)}}`;

const value = (item) =>
  `Value{name:${string(item.name)},description:${description(item)}}`;

const tag = (item) =>
  `Tag{name:${string(item.name)},description:${description(item)},void_element:${item.void === true},attributes:${slice(item.attributes ?? [], attribute)},references:${references(item)},browsers:${browsers(item)},status:${status(item)}}`;

const valueSet = (item) =>
  `ValueSet{name:${string(item.name)},values:${slice(item.values, value)}}`;

// The port of `generateDocumentation` is checked against the function itself,
// on every entry the data holds. The three baseline images are substituted for
// a token on both sides: they are pinned by the SHA-256 in the header and by
// their own equality test, and inlining ~1.5 KB of base64 per row would make
// this file 1.2 MB.
function writeOracle(providerPath, htmlData, images) {
  const require = createRequire(import.meta.url);
  const { generateDocumentation } = require(providerPath);
  const tokens = Object.entries(images);
  const render = (item, markdown) => {
    const result = generateDocumentation(item, {}, markdown);
    if (!result) {
      return null;
    }
    let value = result.value;
    for (const [name, uri] of tokens) {
      value = value.split(uri).join(`<${name}>`);
    }
    return value;
  };
  const rows = {};
  const record = (key, item) => {
    rows[key] = [render(item, true), render(item, false)];
  };
  // Three tags declare the same attribute name twice with different content
  // (`link`/`img` `importance`, `iframe` `allowpaymentrequest`), so the index
  // is part of the key.
  htmlData.tags.forEach((tag) => {
    record(`tag:${tag.name}`, tag);
    (tag.attributes ?? []).forEach((attribute, index) => {
      record(`tag:${tag.name}/attr:${index}:${attribute.name}`, attribute);
    });
  });
  htmlData.globalAttributes.forEach((attribute, index) => {
    record(`global:${index}:${attribute.name}`, attribute);
  });
  fs.mkdirSync(path.dirname(ORACLE), { recursive: true });
  // The images are carried verbatim rather than as the token: substituting
  // them on both sides makes a corrupted constant replace itself, and the
  // comparison stays green.
  fs.writeFileSync(ORACLE, `${JSON.stringify({ images, entries: rows })}\n`);
  process.stdout.write(
    `${path.relative(ROOT, ORACLE)}: ${Object.keys(rows).length} entries\n`,
  );
}

function main() {
  const flag = process.argv.indexOf("--package-root");
  const override = flag === -1 ? undefined : path.resolve(process.argv[flag + 1]);
  const version = lockedVersion();
  const root = packageRoot(override, version);
  const dataPath = path.join(root, DATA_FILE);
  const providerPath = path.join(root, PROVIDER_FILE);
  const require = createRequire(import.meta.url);
  const { htmlData } = require(dataPath);
  const { BaselineImages } = require(providerPath);

  const header = `//! HTML tag and attribute data, generated — do not edit.
//!
//! Source: ${PACKAGE}@${version} (MIT), the build \`package.json\` \`main\`
//! resolves to, which is the one the official language server loads.
//!
//!   ${DATA_FILE}
//!     sha256 ${digest(dataPath)}
//!   ${PROVIDER_FILE}
//!     sha256 ${digest(providerPath)}
//!
//! Regenerate with \`node scripts/dev/generate-html-data.mjs\`.

`;

  const body = `/// A documentation link \`generateDocumentation\` renders after the prose.
pub struct Reference {
    pub name: &'static str,
    pub url: &'static str,
}

/// \`status.baseline\`, which is \`false\` rather than a string when a feature is
/// not baseline at all.
pub enum Baseline {
    Limited,
    Low,
    High,
}

pub struct Status {
    pub baseline: Baseline,
    pub low_date: Option<&'static str>,
    pub high_date: Option<&'static str>,
}

pub struct Value {
    pub name: &'static str,
    pub description: Option<&'static str>,
}

pub struct Attribute {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub value_set: Option<&'static str>,
    pub references: &'static [Reference],
    pub browsers: &'static [&'static str],
    pub status: Option<Status>,
}

pub struct Tag {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub void_element: bool,
    pub attributes: &'static [Attribute],
    pub references: &'static [Reference],
    pub browsers: &'static [&'static str],
    pub status: Option<Status>,
}

pub struct ValueSet {
    pub name: &'static str,
    pub values: &'static [Value],
}

pub const VERSION: &str = ${string(String(htmlData.version))};

pub const BASELINE_LIMITED_IMAGE: &str = ${string(BaselineImages.BASELINE_LIMITED)};
pub const BASELINE_LOW_IMAGE: &str = ${string(BaselineImages.BASELINE_LOW)};
pub const BASELINE_HIGH_IMAGE: &str = ${string(BaselineImages.BASELINE_HIGH)};

pub const TAGS: &[Tag] = ${slice(htmlData.tags, tag)};

pub const GLOBAL_ATTRIBUTES: &[Attribute] = ${slice(htmlData.globalAttributes, attribute)};

pub const VALUE_SETS: &[ValueSet] = ${slice(htmlData.valueSets, valueSet)};
`;

  fs.mkdirSync(path.dirname(OUTPUT), { recursive: true });
  fs.writeFileSync(OUTPUT, header + body);
  writeOracle(providerPath, htmlData, BaselineImages);
  process.stdout.write(
    `${path.relative(ROOT, OUTPUT)}: ${htmlData.tags.length} tags, ${htmlData.globalAttributes.length} global attributes, ${htmlData.valueSets.length} value sets from ${PACKAGE}@${version}\n`,
  );
}

main();
