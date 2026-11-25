// PIXLZR WASM demo glue — vanilla ES module, no bundler.
//
// Everything runs locally: the dropped file is decoded by the browser
// (`createImageBitmap`), encoded/decoded by `pixlzr-core` compiled to WASM,
// and rendered back to a canvas. No network requests, no uploads.

const BUILD_HINT =
  "wasm-pack build crates/pixlzr-core --target web --out-dir ../../www/pkg -- --features wasm";

const MAX_PIXELS = 16_777_216; // matches pixlzr-core's MAX_PIXELS (4096x4096)

const els = {
  drop: document.getElementById("drop"),
  file: document.getElementById("file"),
  q: document.getElementById("q"),
  qv: document.getElementById("qv"),
  error: document.getElementById("error"),
  stats: document.getElementById("stats"),
  origCanvas: document.getElementById("orig"),
  decCanvas: document.getElementById("dec"),
  origMeta: document.getElementById("orig-meta"),
  decMeta: document.getElementById("dec-meta"),
  dlPixlzr: document.getElementById("dl-pixlzr"),
  dlPng: document.getElementById("dl-png"),
};

let codec = null;
let busy = false;
let urls = [];

function qualityFromSlider() {
  return Math.min(100, Math.max(40, Math.round(Number(els.q.value) * 100)));
}

function updateQualityLabel() {
  els.qv.textContent = `${Number(els.q.value).toFixed(2)} (q${qualityFromSlider()})`;
}

function showError(message) {
  els.error.textContent = message;
}

function clearError() {
  els.error.textContent = "";
}

function setBusy(value) {
  busy = value;
  els.file.disabled = value || !codec;
  els.q.disabled = value || !codec;
  els.drop.classList.toggle("busy", value);
  els.drop.setAttribute("aria-busy", String(value));
}

function formatKB(bytes) {
  const kb = bytes / 1024;
  return kb >= 100 ? kb.toFixed(0) : kb.toFixed(1);
}

function drawCanvas(canvas, imageData) {
  canvas.width = imageData.width;
  canvas.height = imageData.height;
  const ctx = canvas.getContext("2d");
  ctx.putImageData(imageData, 0, 0);
}

function resetOutputs() {
  for (const url of urls) URL.revokeObjectURL(url);
  urls = [];
  for (const link of [els.dlPixlzr, els.dlPng]) {
    link.hidden = true;
    link.removeAttribute("href");
  }
}

function setDownload(link, blob, filename) {
  const url = URL.createObjectURL(blob);
  urls.push(url);
  link.href = url;
  link.download = filename;
  link.hidden = false;
}

async function loadBitmap(file) {
  if (typeof createImageBitmap === "function") {
    return await createImageBitmap(file);
  }
  // Fallback for browsers without createImageBitmap.
  const url = URL.createObjectURL(file);
  try {
    const img = new Image();
    img.src = url;
    await img.decode();
    return img;
  } finally {
    URL.revokeObjectURL(url);
  }
}

async function fileToPixels(file) {
  const source = await loadBitmap(file);
  try {
    const width = source.width;
    const height = source.height;
    if (!width || !height) throw new Error("image has no pixels");
    if (width * height > MAX_PIXELS) {
      throw new Error(
        `image too large (${width}×${height}); the demo limit is ${MAX_PIXELS} pixels (e.g. 4096×4096)`
      );
    }

    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    ctx.drawImage(source, 0, 0);
    return { width, height, imageData: ctx.getImageData(0, 0, width, height) };
  } finally {
    if (typeof source.close === "function") source.close();
  }
}

function canvasToPngBlob(canvas) {
  return new Promise((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob) resolve(blob);
      else reject(new Error("canvas.toBlob() returned null"));
    }, "image/png");
  });
}

async function handleFile(file) {
  if (!file || busy) return;

  const looksLikeImage =
    file.type.startsWith("image/") ||
    (!file.type && /\.(png|jpe?g|webp|gif|bmp)$/i.test(file.name || ""));
  if (!looksLikeImage) {
    showError(`Ignored ${file.name || "dropped item"} — not an image (${file.type || "unknown type"}).`);
    return;
  }
  if (!codec) {
    showError(`WASM not built — run: ${BUILD_HINT}`);
    return;
  }

  setBusy(true);
  clearError();
  resetOutputs();
  els.stats.textContent = "Working…";

  try {
    const { width, height, imageData } = await fileToPixels(file);
    drawCanvas(els.origCanvas, imageData);
    els.origMeta.textContent = `${width}×${height} · ${formatKB(file.size)} KB`;

    const pixels = new Uint8Array(
      imageData.data.buffer,
      imageData.data.byteOffset,
      imageData.data.byteLength
    );

    const encodeStart = performance.now();
    const blob = codec.encode_rgba(width, height, pixels, qualityFromSlider());
    const encodeEnd = performance.now();

    const decodeStart = performance.now();
    const rgba = codec.decode_rgba(blob);
    const decodeEnd = performance.now();

    const [decWidth, decHeight] = Array.from(codec.image_info(blob));
    if (decWidth !== width || decHeight !== height) {
      throw new Error(`decoded dimensions ${decWidth}×${decHeight} != ${width}×${height}`);
    }
    if (rgba.length !== width * height * 4) {
      throw new Error(`decoded ${rgba.length} bytes, expected ${width * height * 4}`);
    }

    const decodedImage = new ImageData(
      new Uint8ClampedArray(rgba.buffer, rgba.byteOffset, rgba.byteLength),
      width,
      height
    );
    drawCanvas(els.decCanvas, decodedImage);
    els.decMeta.textContent = `${decWidth}×${decHeight} · ${formatKB(blob.length)} KB`;

    const ratio = file.size / blob.length;
    els.stats.textContent =
      `Encode ${(encodeEnd - encodeStart).toFixed(1)} ms · ` +
      `Decode ${(decodeEnd - decodeStart).toFixed(1)} ms · ` +
      `${formatKB(file.size)} KB -> ${formatKB(blob.length)} KB (${ratio.toFixed(1)}:1)`;
    els.stats.title =
      `raw RGBA ${formatKB(width * height * 4)} KB, quality ${qualityFromSlider()}, ` +
      `codec ${codec.version()}`;

    setDownload(els.dlPixlzr, new Blob([blob], { type: "application/octet-stream" }), "output.pixlzr");
    setDownload(els.dlPng, await canvasToPngBlob(els.decCanvas), "restored.png");
  } catch (error) {
    showError(error instanceof Error ? error.message : String(error));
    els.stats.textContent = "Encode — · Decode — · Ratio —";
  } finally {
    setBusy(false);
  }
}

function wireDropzone() {
  for (const event of ["dragenter", "dragover"]) {
    els.drop.addEventListener(event, (e) => {
      e.preventDefault();
      els.drop.classList.add("over");
    });
  }
  for (const event of ["dragleave", "drop"]) {
    els.drop.addEventListener(event, (e) => {
      e.preventDefault();
      els.drop.classList.remove("over");
    });
  }
  els.drop.addEventListener("drop", (e) => {
    handleFile(e.dataTransfer?.files?.[0]);
  });
  // Dropping outside the zone must not navigate away / open the file.
  window.addEventListener("dragover", (e) => e.preventDefault());
  window.addEventListener("drop", (e) => e.preventDefault());

  els.file.addEventListener("change", async () => {
    const file = els.file.files?.[0];
    els.file.value = "";
    await handleFile(file);
  });
  els.q.addEventListener("input", updateQualityLabel);
  els.drop.addEventListener("click", (e) => {
    if (e.target === els.drop && !busy && codec) els.file.click();
  });
}

async function boot() {
  updateQualityLabel();
  wireDropzone();

  try {
    const module = await import("../pkg/pixlzr_core.js");
    await module.default();
    codec = module;
    console.info(`pixlzr-core ${codec.version()} ready`);
  } catch (error) {
    codec = null;
    els.q.disabled = true;
    els.file.disabled = true;
    els.drop.classList.add("disabled");
    els.stats.textContent = "WASM not built — demo disabled.";
    showError(`WASM not built — run: ${BUILD_HINT} (${error instanceof Error ? error.message : error})`);
  }
}

boot();
