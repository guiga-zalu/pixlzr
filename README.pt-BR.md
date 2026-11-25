# PIXLZR — Codec de Imagem Sensível à Densidade em Rust + WASM

[English](README.md) | Português (BR)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![CI](https://github.com/guiga-zalu/pixlzr/actions/workflows/ci.yml/badge.svg)
![Pages](https://github.com/guiga-zalu/pixlzr/actions/workflows/pages.yml/badge.svg)
![Benchmarks](https://img.shields.io/badge/benchmarks-pending-lightgrey)

> **Resumo em uma linha:** um codec de imagem com quantização adaptativa por bloco — blocos DCT 8×8 cujo passo de quantização é escolhido a partir da densidade do gradiente de luma local. Blocos suaves mantêm passos mais finos (banding é visível neles); blocos movimentados são quantizados mais grosseiramente (o ruído é mascarado). Formato próprio, núcleo em Rust, CLI e demo WASM no navegador que realmente codifica e reconstrói a imagem.

**🎮 Demo ao vivo (WASM):** https://guiga-zalu.github.io/pixlzr/ — arraste um PNG/JPEG → codifica em `.pixlzr` → decodifica → antes/depois com razão e tempos. Tudo roda localmente; nada é enviado. Instruções de build/execução abaixo.

> **GIF da demo:** ainda não gravado. O roteiro e a especificação do arquivo estão em [`assets/demo-script.md`](assets/demo-script.md) e [`assets/demo-gif-spec.md`](assets/demo-gif-spec.md).

---

## O que é o PIXLZR?

O PIXLZR v1 é um codec de imagem pequeno, honesto e com poucas dependências:

- **Formato v1** — cabeçalho de 27 bytes + passos de quantização por bloco + um fluxo zlib. Especificação byte a byte: [`docs/format.md`](docs/format.md).
- **Transformada real** — YCoCg inteiro reversível, DCT-II 8×8 ortonormal, varredura zigzag, coeficientes `i16`.
- **Quantização adaptativa à densidade** — o passo de cada bloco vem da média do gradiente absoluto de luma naquele bloco: blocos suaves recebem passos mais finos (banding é visível em áreas planas), blocos texturizados recebem passos mais grossos (o ruído de quantização é mascarado pela textura). A fórmula exata está na especificação e no código.
- **API nativa + CLI** — `encode_rgba` / `decode_rgba` / `inspect`, além de `pixlzr encode|decode|info`.
- **Demo WASM** — o mesmo código Rust compilado para `wasm32-unknown-unknown`, controlado por um front end em ES module puro (sem bundler).
- **Testes + benchmarks** — asserções de PSNR/razão no round-trip, testes de rejeição de cabeçalho e benchmarks Criterion.

**Explicitamente ainda não implementado** (não presuma o contrário):

- Multithreading / Rayon. Tanto o caminho nativo quanto a demo WASM são **single-thread**. O WASM é single-thread porque a demo não usa `SharedArrayBuffer`.
- O filtro de vídeo a 60 fps, o patch para o visualizador de imagens e o modelo de densidade aprendido α/β/γ das notas de design antigas.
- SSIM, métricas perceptuais e comparações em qualidade equiparada contra JPEG/PNG/WebP.
- Modo sem perdas, codificação de alfa, metadados, decodificação em streaming e CRC.

Os documentos de design antigos ([`docs/architecture.md`](docs/architecture.md), [`docs/wasm-demo-spec.md`](docs/wasm-demo-spec.md)) são mantidos como contexto, com notas de status onde descrevem planos e não código entregue.

---

## Início rápido

```bash
# Requer Rust estável. Nenhuma biblioteca de imagem do sistema para o núcleo.
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features

# CLI (PNG <-> .pixlzr)
cargo build --release --features cli
./target/release/pixlzr encode input.png output.pixlzr --quality 80
./target/release/pixlzr info output.pixlzr
./target/release/pixlzr decode output.pixlzr restored.png

# Benchmarks (Criterion)
cargo bench --features bench
```

### Demo WASM localmente

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked   # ou use o pacote da sua distro

wasm-pack build crates/pixlzr-core --target web --out-dir ../../www/pkg -- --features wasm
python3 -m http.server --directory www 8000
# abra http://localhost:8000 e arraste uma imagem
```

Se `www/pkg/` não existir, a página da demo mostra o comando de build em vez de um stack trace.

---

## Arquitetura

```
RGBA8 ──► YCoCg reversível ──► DCT-II 8×8 ──► quantização (passo vindo da densidade local)
                                                         │
                                                         ▼
                                          coeficientes i16 em zigzag ──► zlib
                                                         │
   .pixlzr  =  cabeçalho + passos por bloco + payload zlib ◄──┘

Decodificação: lê cabeçalho → infla → desquantiza → IDCT → YCoCg inverso → clamp → RGBA8
```

Tudo vive em `crates/pixlzr-core`; o crate não faz I/O (a CLI faz).
As exportações WASM são wrappers finos sobre as mesmas funções nativas, então a demo
não consegue divergir do codec. Detalhes: [`docs/architecture.md`](docs/architecture.md).

---

## Especificação do formato

Cabeçalho (little-endian):

| Campo | Tamanho |
|---|---|
| magic `PIXLZR` + versão `1` | 7 bytes |
| flags (`0`) | 2 bytes |
| width, height | 4 bytes cada |
| tamanho do bloco (`8`), qualidade | 1 byte cada |
| blocks_x, blocks_y | 4 bytes cada |
| passos de quantização por bloco | `blocks_x * blocks_y` bytes |
| fluxo zlib de coeficientes `i16` em zigzag | restante do arquivo |

Especificação normativa completa, incluindo a fórmula de densidade/quantização, tabela zigzag,
regra de padding, exemplo hexadecimal resolvido e limitações: [`docs/format.md`](docs/format.md).

---

## Benchmarks

> **Nenhum número de benchmark é publicado ainda.** A tabela abaixo está
> intencionalmente vazia; as linhas só serão preenchidas a partir de execuções do
> Criterion em hardware fixo, em qualidade equiparada, com a saída bruta commitada.
> Os autotestes exigem `PSNR > 30 dB` e `bruto/codificado > 2:1` na qualidade 80
> em um gradiente sintético.

| Dataset | PIXLZR (razão em qualidade equiparada) | JPEG | PNG | WebP | Enc ms | Dec ms | Notas |
|---|---|---|---|---|---|---|---|
| Kodak (24× 768×512) | _medição pendente_ | — | — | — | — | — | comparações de baseline não implementadas |
| CLIC valid | _medição pendente_ | — | — | — | — | — | dataset não incluído |
| Sintético 512×512 (gradiente + ruído LCG) | _medição pendente_ | — | — | — | — | — | `cargo bench --features bench` |

Reproduza o harness:

```bash
cargo bench --features bench
# o criterion imprime estimativas de média/mediana; JSON bruto em target/criterion/
```

A entrada do benchmark é gerada de forma determinística em `benches/codec.rs`
(gradiente + ruído LCG), então execuções são comparáveis entre máquinas. Template
para a futura tabela em qualidade equiparada: [`benches/bench-table.md`](benches/bench-table.md);
notas do harness: [`benches/README.md`](benches/README.md).

---

## Roadmap

- [x] 2021–2022 — experimentos com função de densidade, porte para Rust
- [x] 2023 — protótipo de formato próprio
- [x] **v1 (estado atual deste repositório)** — formato + codec reais, YCoCg reversível,
  quantização adaptativa por bloco, API nativa, CLI (`encode`/`decode`/`info`),
  exportações WASM, demo drag-drop, testes de round-trip, benchmarks Criterion, CI + Pages
- [ ] Modo sem perdas e codificação de alfa
- [ ] Benchmarks em qualidade equiparada vs JPEG/PNG/WebP (gate SSIM/PSNR)
- [ ] Codificação/decodificação multithread (Rayon) e decodificador em streaming
- [ ] Codificação de entropia melhor (rANS/aritmética) e blocos adaptativos
- [ ] Integração com visualizador para `.pixlzr`

---

## Contribuindo

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo bench --no-run --features bench
```

O CI roda as mesmas verificações (`.github/workflows/ci.yml`). Mudanças de formato
devem atualizar [`docs/format.md`](docs/format.md) no mesmo PR.

---

## Licença

Licença dupla sob **MIT OR Apache-2.0** — veja [`LICENSE-MIT`](LICENSE-MIT) e [`LICENSE-APACHE`](LICENSE-APACHE).

`SPDX-License-Identifier: MIT OR Apache-2.0` — Copyright (c) 2021–2026 Guilherme Alves Ceobaniuk Zaluchi

---

## Citação / Autor

Guilherme Zaluchi — [@guiga-zalu](https://github.com/guiga-zalu) · [LinkedIn](https://linkedin.com/in/guilherme-alves-c-zaluchi) · Ribeirão Pires, SP (Remoto)
