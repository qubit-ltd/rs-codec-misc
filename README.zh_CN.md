# Qubit Misc Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec-misc/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec-misc/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec-misc/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec-misc/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec-misc.svg?color=blue)](https://crates.io/crates/qubit-codec-misc)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

为 Rust 应用提供可复用的字节与文本编解码器。

## 概述

Qubit Misc Codec 提供小而明确的编解码器，用于 Qubit Rust crate 和应用中常见的稳定字节与文本编码。它保持 Rust API 轻量、类型明确且符合 Rust 习惯：常见场景直接使用具体方法，泛型边界再使用 trait。

本 crate 聚焦具有清晰线格式语义的文本编码：

- 十六进制字节串
- Base64 字节串
- C 整数字面量片段
- C 字符串字面量字节片段
- percent-encoded UTF-8 文本
- `application/x-www-form-urlencoded` UTF-8 文本片段

它不会替代 Rust 的 `Display`、`FromStr`、`TryFrom` 或 `serde`，这些 API 仍然适合普通对象转换。

## 设计目标

- **语义明确**：每个 codec 都说明字母表、分隔符、填充和解码规则。
- **API 表面小**：优先提供直接的 `encode` 和 `decode` 方法，泛型场景再使用 trait。
- **无隐藏 Panic**：畸形输入返回 `MiscCodecError`，不直接 panic。
- **Trait 分层**：`Codec` 面向低层单值或 quantum 转换，`ValueEncoder` 和 `ValueDecoder`
  仍是自有完整值便捷 trait。通用 adapter、buffered engine 和策略 hook 位于
  `qubit-codec`；围绕这些 codec 构建自定义 adapter 时请直接从 core crate 引入。
- **实现可复用**：常用编码集中在一个 crate，避免下游重复实现。
- **依赖最少化**：只在确有价值时依赖维护良好的第三方 crate。

## 特性

### 🔡 **十六进制字节**

- **默认小写**：`HexCodec::new()` 生成连续小写十六进制文本。
- **大写模式**：`HexCodec::upper()` 或 `with_uppercase(true)` 生成大写字符。
- **可选整体前缀**：在整个编码值前添加并要求前缀，例如 `0x`。
- **可选逐字节前缀**：在每个编码字节前添加并要求 byte 前缀，例如 `0x`。
- **可选分隔符**：在字节之间写入并接受分隔符，例如 `:` 或空格。
- **空白处理**：解码时可选择忽略 ASCII 空白字符。
- **前缀大小写处理**：解码匹配已配置前缀时，可选择忽略 ASCII 大小写。
- **缓冲区 API**：`encode_into` 和 `decode_into` 可追加写入已有缓冲区。

### 🔐 **Base64 字节**

- **标准字母表**：支持带 padding 和无 padding 的标准 Base64。
- **URL 安全字母表**：支持带 padding 和无 padding 的 URL-safe Base64。
- **Quantum Core**：`Base64QuantumCodec` 处理完整的三字节到四 unit Base64
  quantum；final padding 留在 facade/transcoder 层。
- **类型化错误**：畸形输入返回 `MiscCodecError::InvalidInput`。

### 🔤 **C 字符串字面量字节**

- **混合文本与转义**：解码 `PK\003\004` 和 `\xd0\xcf` 这样的片段。
- **C 转义支持**：支持简单转义、八进制转义、十六进制转义和 universal byte escape。
- **字节输出**：直接解码为原始字节，不要求 UTF-8。

### 🔢 **C 整数字面量**

- **进制识别**：解码十进制、八进制和 `0x`/`0X` 十六进制整数字面量。
- **无符号输出**：将非负整数字面量片段解析为 `u64`。
- **精确错误**：非法数字会携带原始输入中的字节位置。
- **Value-token decode**：仍作为 `ValueDecoder<str>` convenience codec，因为整数字面量的
  编码策略和 token 边界尚不属于当前低层单值抽象。

### 🌐 **Percent-Encoding**

- **UTF-8 文本**：编码和解码 UTF-8 字符串。
- **RFC 3986 unreserved 集合**：ASCII 字母、数字、`-`、`.`、`_` 和 `~` 保持不变。
- **大写转义**：写出 `%2F`、`%E4` 这样的 percent escape。
- **畸形转义检测**：报告截断或非法的 `%XX` 序列。

### 📝 **Form URL Encoding**

- **表单片段 codec**：处理 `application/x-www-form-urlencoded` 文本片段。
- **空格使用加号**：空格编码为 `+`，解码时 `+` 还原为空格。
- **Percent 兼容**：复用与 `PercentCodec` 相同的 UTF-8 和 `%XX` 校验行为。

### 🎯 **聚焦的公开 API**

- **`ValueEncoder<Input>`**：将借用输入编码为关联输出类型。
- **`ValueDecoder<Input>`**：将借用输入解码为关联输出类型。
- **`Codec`（关联类型 `Value` 与 `Unit`）**：低层 unsafe trait，用于在调用方提供的 unit 缓冲区上处理一个值或一个 codec quantum。
- **`CodecValueEncoder<C>` / `CodecTranscodeEncoder<C>` /
  `CodecTranscodeDecoder<C>`**：由 `qubit-codec` 提供的默认 value 和
  buffered adapter。
- **`TranscodeEncodeEngine` / `TranscodeEncodeHooks` /
  `TranscodeDecodeEngine` / `TranscodeDecodeHooks`**：由 `qubit-codec` 提供、
  面向自定义策略 adapter 的可复用 buffered engine 与 hook。
- **`MiscCodecError` / `MiscCodecResult`**：内置 codec 的公共错误与结果类型。

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
qubit-codec-misc = "0.1"
```

## 快速开始

### 十六进制字节

```rust
use qubit_codec_misc::HexCodec;

fn main() {
    let codec = HexCodec::upper()
        .with_prefix("0x")
        .with_separator(" ");

    let encoded = codec.encode(&[0x1f, 0x8b, 0x00, 0xff]);
    assert_eq!("0x1F 8B 00 FF", encoded);

    let decoded = codec
        .decode("0x1F 8B 00 FF")
        .expect("hex text should decode");
    assert_eq!(vec![0x1f, 0x8b, 0x00, 0xff], decoded);
}
```

### Base64 字节

```rust
use qubit_codec_misc::Base64Codec;

fn main() {
    let codec = Base64Codec::standard();

    let encoded = codec.encode(b"hello");
    assert_eq!("aGVsbG8=", encoded);

    let decoded = codec
        .decode("aGVsbG8=")
        .expect("Base64 text should decode");
    assert_eq!(b"hello".to_vec(), decoded);
}
```

### 无 Padding 的 URL-Safe Base64

```rust
use qubit_codec_misc::Base64Codec;

fn main() {
    let codec = Base64Codec::url_safe_no_pad();

    let encoded = codec.encode(&[251, 255, 239]);
    assert_eq!("-__v", encoded);

    let decoded = codec
        .decode("-__v")
        .expect("URL-safe Base64 text should decode");
    assert_eq!(vec![251, 255, 239], decoded);
}
```

### C 字符串字面量字节

```rust
use qubit_codec_misc::CStringLiteralCodec;

fn main() {
    let codec = CStringLiteralCodec::new();

    let decoded = codec
        .decode(r"PK\003\004")
        .expect("C string literal should decode");
    assert_eq!(b"PK\x03\x04".to_vec(), decoded);

    let encoded = codec.encode(&[0xd0, 0xcf, 0x11, 0xe0]);
    assert_eq!(r"\xD0\xCF\x11\xE0", encoded);
}
```

### C 整数字面量

```rust
use qubit_codec_misc::CIntegerLiteralCodec;

fn main() {
    let codec = CIntegerLiteralCodec::new();

    assert_eq!(123, codec.decode("123").expect("decimal should decode"));
    assert_eq!(83, codec.decode("0123").expect("octal should decode"));
    assert_eq!(
        0xbeef_c0de,
        codec.decode("0xBEEFC0DE").expect("hex should decode")
    );
}
```

### Percent-Encoding UTF-8 文本

```rust
use qubit_codec_misc::PercentCodec;

fn main() {
    let codec = PercentCodec::new();

    let encoded = codec.encode("a b/中");
    assert_eq!("a%20b%2F%E4%B8%AD", encoded);

    let decoded = codec
        .decode("a%20b%2F%E4%B8%AD")
        .expect("percent-encoded text should decode");
    assert_eq!("a b/中", decoded);
}
```

### Form URL Encoding

```rust
use qubit_codec_misc::FormUrlencodedCodec;

fn main() {
    let codec = FormUrlencodedCodec::new();

    let encoded = codec.encode("name=Qubit Codec");
    assert_eq!("name%3DQubit+Codec", encoded);

    let decoded = codec
        .decode("name%3DQubit+Codec")
        .expect("form-url-encoded text should decode");
    assert_eq!("name=Qubit Codec", decoded);
}
```

### 泛型 Trait 用法

当应用代码只依赖“具备某种编码能力”，而不想依赖具体 codec 类型时，可以使用 trait。

```rust
use qubit_codec_misc::{
    MiscCodecError,
    ValueEncoder,
    HexCodec,
};

fn encode_payload<C>(codec: &C, payload: &[u8]) -> Result<String, MiscCodecError>
where
    C: ValueEncoder<[u8], Output = String, Error = MiscCodecError>,
{
    codec.encode(payload)
}

fn main() {
    let text = encode_payload(&HexCodec::new(), &[0xab, 0xcd])
        .expect("hex encoding should not fail");
    assert_eq!("abcd", text);
}
```

## API 参考

### Trait 操作

| Trait | 方法 | 描述 |
|-------|------|------|
| `ValueEncoder<Input>` | `encode(&Input)` | 将借用输入编码为关联输出类型 |
| `ValueDecoder<Input>` | `decode(&Input)` | 将借用输入解码为关联输出类型 |
| `Codec`（关联类型 `Value` 与 `Unit`） | `decode`, `encode` | 在调用方提供的 unit 缓冲区上转换一个值或一个 codec quantum |

低层 `Codec` 实现刻意排除 facade 关注点：十六进制 prefix/separator、UTF-8
`String` 校验和 Base64 final padding 都由 value helper 或后续 buffered 层处理。

### `HexCodec` 操作

| 方法 | 描述 |
|------|------|
| `new()` | 创建无前缀、无分隔符的小写 codec |
| `upper()` | 创建无前缀、无分隔符的大写 codec |
| `with_uppercase(enabled)` | 配置字符大小写 |
| `with_prefix(prefix)` | 添加并要求整体前缀，例如 `0x1F8B` |
| `with_byte_prefix(prefix)` | 在每个 byte 前添加并要求前缀，例如 `0x1F 0x8B` |
| `with_separator(separator)` | 在字节之间添加并接受分隔符 |
| `with_ignored_ascii_whitespace(enabled)` | 解码时忽略 ASCII 空白字符 |
| `with_ignore_prefix_case(enabled)` | 解码匹配已配置前缀时忽略 ASCII 大小写 |
| `encode(bytes)` | 将字节编码为十六进制文本 |
| `encode_into(bytes, output)` | 将编码文本追加到已有 `String` |
| `decode(text)` | 将十六进制文本解码为字节 |
| `decode_into(text, output)` | 将解码字节追加到已有 `Vec<u8>` |

### `Base64Codec` 操作

| 方法 | 字母表 | Padding | 描述 |
|------|--------|---------|------|
| `standard()` | 标准 | 有 | 创建标准 Base64 codec |
| `standard_no_pad()` | 标准 | 无 | 创建无 padding 的标准 Base64 codec |
| `url_safe()` | URL-safe | 有 | 创建 URL-safe Base64 codec |
| `url_safe_no_pad()` | URL-safe | 无 | 创建无 padding 的 URL-safe Base64 codec |
| `encode(bytes)` | 已配置 | 已配置 | 将字节编码为 Base64 文本 |
| `decode(text)` | 已配置 | 已配置 | 将 Base64 文本解码为字节 |

### `Base64QuantumCodec` 操作

| 方法 | 字母表 | Units | 描述 |
|------|--------|-------|------|
| `standard()` | 标准 | 4 | 创建标准 Base64 quantum codec |
| `url_safe()` | URL-safe | 4 | 创建 URL-safe Base64 quantum codec |
| `Codec<Value = [u8; 3], Unit = u8>` | 已配置 | 4 | 编码或解码一个完整 Base64 quantum，不处理 padding finalization |

### `CStringLiteralCodec` 操作

| 方法 | 描述 |
|------|------|
| `new()` | 创建 C 字符串字面量字节 codec |
| `encode(bytes)` | 将字节编码为 C 字符串字面量片段 |
| `decode(text)` | 将 C 字符串字面量片段解码为字节 |

### `CIntegerLiteralCodec` 操作

| 方法 | 描述 |
|------|------|
| `new()` | 创建 C 整数字面量解码器 |
| `decode(text)` | 将非负 C 整数字面量片段解码为 `u64` |

`CIntegerLiteralCodec` 刻意保留为 value-token decoder，暂不实现 `Codec<Value = u64, Unit = u8>`。
原因是这会提前承诺 token 边界和 encode 格式策略，而这些策略应位于单值 core 之上。

### 文本 Codec 操作

| 类型 | 方法 | 描述 |
|------|------|------|
| `PercentCodec` | `new()` | 创建 percent codec |
| `PercentCodec` | `encode(text)` | 使用 percent encoding 编码 UTF-8 文本 |
| `PercentCodec` | `decode(text)` | 解码 percent-encoded UTF-8 文本 |
| `FormUrlencodedCodec` | `new()` | 创建 form-url-encoded codec |
| `FormUrlencodedCodec` | `encode(text)` | 编码 UTF-8 文本，并将空格写为 `+` |
| `FormUrlencodedCodec` | `decode(text)` | 解码 UTF-8 文本，并将 `+` 视为空格 |

## 错误处理

内置 decoder 返回 `MiscCodecResult<T>`，也就是 `Result<T, MiscCodecError>`。

| 错误 | 含义 |
|------|------|
| `MissingPrefix` | 配置了整体或逐字节十六进制前缀，但输入缺少该前缀 |
| `InvalidDigit` | 输入包含不符合指定进制的数字字符 |
| `InvalidLength` | 输入长度不满足 codec 要求 |
| `InvalidEscape` | 输入包含畸形或不支持的转义序列 |
| `InvalidCharacter` | 输入包含当前位置不允许的字符 |
| `InvalidInput` | 输入被 codec 专属校验拒绝 |
| `InvalidUtf8` | 解码后的字节不是合法 UTF-8 |

## 性能考虑

Codec 实现直接操作借用的 byte slice 或字符串，只在目标格式确实需要时返回 owned output。
配置存放在小型值类型中，泛型 trait 用法也不要求动态分发。

## 测试与代码覆盖率

本项目通过 `tests/` 下的集成测试覆盖 codec 行为。

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行覆盖率报告
./coverage.sh

# 生成文本格式报告
./coverage.sh text

# 对齐 CI 要求
./align-ci.sh

# 运行 CI 检查（格式化、clippy、测试、覆盖率、安全审计）
./ci-check.sh
```

## 依赖项

运行时依赖保持很少：

- `base64` 提供 Base64 engine。
- `thiserror` 提供公共错误类型实现。

## 许可证

Copyright (c) 2026. Haixing Hu.

根据 Apache 许可证 2.0 版（"许可证"）授权；
除非遵守许可证，否则您不得使用此文件。
您可以在以下位置获取许可证副本：

    http://www.apache.org/licenses/LICENSE-2.0

除非适用法律要求或书面同意，否则根据许可证分发的软件
按"原样"分发，不附带任何明示或暗示的担保或条件。
有关许可证下的特定语言管理权限和限制，请参阅许可证。

完整的许可证文本请参阅 [LICENSE](LICENSE)。

## 贡献

欢迎贡献！请随时提交 Pull Request。

### 开发指南

- 遵循 Rust API 指南。
- 保持测试全面且稳定。
- 为公共 API 和行为变化编写文档。
- 提交 PR 前确保所有检查通过。

## 作者

**胡海星**

## 相关项目

Qubit 旗下的更多 Rust 库发布在 GitHub 组织 [qubit-ltd](https://github.com/qubit-ltd)。

---

仓库地址：[https://github.com/qubit-ltd/rs-codec-misc](https://github.com/qubit-ltd/rs-codec-misc)
