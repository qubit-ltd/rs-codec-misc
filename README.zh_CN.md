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
- **命名配置**：当多个 codec 共享同一策略时，可通过 `HexCodecConfig` 和
  `HexCodec::from_config` 复用配置。
- **Hex 缓冲区 API**：`HexCodec::encode_into` 和 `HexCodec::decode_into`
  可追加写入已有缓冲区。

空白处理规则取决于是否配置了非空分隔符。有分隔符时，同一字节的两位十六进制数字
必须连续；例如，分隔符为 `:` 时会拒绝 `1 f:8b`。没有非空分隔符时，启用忽略空白
可接受字节内部空白，例如逐字节前缀为 `0x` 时可以解析 `0x1 f`。完整的结构标记
优先于额外空白：启用忽略 ASCII 空白且分隔符为 ` : ` 时，`1f\t : 8b` 可解码为
`[0x1f, 0x8b]`。此处的 `\t` 表示实际制表符。非纯空白分隔符必须精确匹配；纯空白
分隔符仍要求字节之间有空白，启用忽略 ASCII 空白后可一并跳过相邻的额外空白。
启用忽略空白不会取消配置的分隔符要求。

### 🔐 **Base64 字节**

- **标准字母表**：支持带 padding 和无 padding 的标准 Base64。
- **URL 安全字母表**：支持带 padding 和无 padding 的 URL-safe Base64。
- **严格 Padding 契约**：每个 facade 只接受自身配置的 padding 形式；`TWFu`
  这类完整组在两种模式下都合法。空白和非规范尾部废弃位均会被拒绝。
- **Quantum Core**：`Base64QuantumCodec` 处理完整的三字节到四 unit Base64
  quantum；final padding 留在 facade/transcoder 层。
- **类型化错误**：畸形输入返回 `MiscCodecError::InvalidBase64`，并保留
  `base64::DecodeError` source、错误分类以及可用的输入位置。
- **三层 API**：`Base64Codec` 编解码完整值；`Base64QuantumCodec` 处理完整的
  三字节/四 unit quantum；`Base64Encoder` 和 `Base64Decoder` 跨多次调用流式处理完整值。
- **独立流状态**：可直接按方向构造 encoder 或 decoder，也可用
  `Base64Codec::encoder()` / `decoder()` 复制其字母表和 padding 配置。每个实例的状态相互独立，
  第一次流式调用前必须先 reset。
- **流式契约**：encoder 在内部最多保留两个输入字节，并在 `finish` 时输出末尾部分组。
  decoder 遇到尚未确认 EOF 的不完整文本尾部时会保留其未消费状态；调用方须保留该输入，
  与新输入合并后重试，或在确认 EOF 后传给 `transcode_eof`。`Complete` 只描述本次调用可见的输入，
  不表示源已到 EOF。
- **容量与错误恢复**：`NeedOutput.required` 表示从推进后的输出位置
  （`output_index + progress.written()`）起算，完成下一步所需的最小总容量；它不是整次调用的输出总量，
  也不是额外需要追加的字节数。保留原缓冲区时，长度至少为该位置加 `required`；换用从索引 0 开始的空缓冲区时，
  至少提供 `required` 个槽位。重试前按 `progress.read()` 推进输入。容量上界是保守值，空输入也适用。
  通用驱动计算推进后的输出位置和所需缓冲区长度时应使用 checked arithmetic，并在分配前处理溢出。
  畸形输入错误可能发生在部分输出之后；此时流进入 Poisoned 状态，复用前必须 reset。
- **错误坐标**：`error.domain_error_ref().and_then(|error| error.input_index())` 是当前调用传入的输入切片内位置，
  不是累计流位置。`MiscCodecError::InvalidBase64` 及其 `base64::DecodeError` source 保留 engine 实际收到的子片段位置；
  该位置可能随批处理、EOF 路径和输出容量变化。engine 未提供精确位置时，外层错误使用出错片段起点。
  需要全流位置的驱动必须自行累计已丢弃的输入。

例如，从索引 2 解码 `__TWFu@AAA`，小输出调用和 EOF 调用的外层错误位置都是 6；内层错误/source 位置分别为 0 和 4，
二者都不是全流位置。

使用流式解码 hook 跳过或替换畸形 percent、form URL 或 C 字符串输入时，codec
报告当前输入 chunk 中可见的非法跨度，因此畸形输入的恢复结果可能受 chunk
边界影响。此说明不适用于有效输入或严格解码。

encoder 的 `NeedOutput.required` 从推进后的输出位置起算。下面的示例保留已有输出，并只重试尚未消费的输入：

本流式示例需要直接声明两个依赖，见[安装](#安装)。

```rust
use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_codec_misc::Base64Encoder;

let mut encoder = Base64Encoder::standard();
encoder.reset(&mut [], 0)?;
let mut output = [0; 4];
let first = encoder.transcode(b"ManMan", 0, &mut output, 0)?;
assert_eq!((first.read(), first.written()), (5, 4));
let required = match first.status() {
    TranscodeStatus::NeedOutput { required } => required.get(),
    status => panic!("expected NeedOutput, got {status:?}"),
};
let mut larger = [0; 8];
larger[..4].copy_from_slice(&output);
let next = encoder.transcode(b"ManMan", first.read(), &mut larger, first.written())?;
assert_eq!(required, 4);
assert_eq!(next.read(), 1);
assert_eq!(encoder.finish(&mut larger, 8)?, 0);
assert_eq!(&larger, b"TWFuTWFu");
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 🔤 **C 字符串字面量字节**

- **混合文本与转义**：解码 `PK\003\004` 和 `\xd0\xcf` 这样的片段。
- **C 转义支持**：支持简单转义、八进制转义、十六进制转义和 universal byte escape。
- **字节输出**：直接解码为原始字节，不要求 UTF-8。

### 🔢 **C 整数字面量**

- **进制识别**：解码十进制、八进制和 `0x`/`0X` 十六进制整数字面量。
- **无符号输出**：将非负整数字面量片段解析为 `u64`。
- **精确错误**：非法数字会携带原始输入中的字节位置。
- **受限 token 契约**：只接受完整的非负 token；不包含符号、整数后缀、数字
  分隔符和 token 流解析。
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
- **WHATWG 字符集合**：仅直接保留 ASCII 字母、数字、`*`、`-`、`.` 和 `_`。
- **Percent 兼容**：复用与 `PercentCodec` 相同的 UTF-8 和 `%XX` 校验行为。

### 🎯 **聚焦的公开 API**

- **`ValueEncoder<Input>`**：将借用输入编码为关联输出类型。
- **`ValueDecoder<Input>`**：将借用输入解码为关联输出类型。
- **[`Codec`](https://docs.rs/qubit-codec/0.17/qubit_codec/trait.Codec.html)（关联类型 `Value` 与 `Unit`）**：低层 trait，包含 unsafe 编解码入口，用于在调用方提供的 unit 缓冲区上处理一个值或一个 codec quantum。调用方必须满足文档规定的索引和容量前置条件；实现使用 `impl Codec`，而非 `unsafe impl Codec`。
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
qubit-codec-misc = "0.8"
```

使用从 `qubit_codec` 导入的 trait、adapter 或流式 API 时，还需将
`qubit-codec` 声明为直接依赖：

```toml
[dependencies]
qubit-codec = "0.17"
qubit-codec-misc = "0.8"
```

## 示例：解析 magic 规则

应用从 XML 中提取属性后，可以组合这些 codec 解析 magic 规则。下面展示的是应用层组合方式，并非完整的 MIME loader：库负责语法和数值解析，应用负责检查实际字段位宽与 mask 长度、将错误关联到源规则，并决定是否加载。解析失败时应拒绝该规则，不要静默保留只解析了一部分的值。

这里的 `number` 是另一个用于演示的数值字段，并不表示字符串 magic 和数值属于同一条 MIME matcher 规则。示例中的 `u16` 仅用于演示，实际位宽应根据目标字段选择。

```rust
use qubit_codec_misc::CIntegerLiteralCodec;
use qubit_codec_misc::CStringLiteralCodec;
use qubit_codec_misc::HexCodec;

fn parse_rule(value: &str, number: &str, mask: &str)
    -> Result<(Vec<u8>, u16, Vec<u8>), String>
{
    let bytes = CStringLiteralCodec::new().decode(value).map_err(|e| e.to_string())?;
    let number = CIntegerLiteralCodec::new().decode(number).map_err(|e| e.to_string())?;
    let number = u16::try_from(number).map_err(|_| "value exceeds 16 bits".to_owned())?;
    let mask = HexCodec::new()
        .with_prefix("0x")
        .with_ignore_prefix_case(true)
        .decode(mask).map_err(|e| e.to_string())?;
    if mask.len() != bytes.len() {
        return Err("mask length must equal magic length".to_owned());
    }
    Ok((bytes, number, mask))
}

fn main() -> Result<(), String> {
    let (bytes, number, mask) = parse_rule(r"PK\003\004", "0x1234", "0XFFFFFFFF")?;
    assert_eq!(b"PK\x03\x04".as_slice(), bytes.as_slice());
    assert_eq!(0x1234, number);
    assert_eq!(vec![0xff; 4], mask);
    assert_eq!(parse_rule("PK", "0x10000", "0xFFFF").unwrap_err(),
        "value exceeds 16 bits");
    assert_eq!(parse_rule("PK", "1", "0xFF").unwrap_err(),
        "mask length must equal magic length");
    Ok(())
}
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

完整值转换与流式转换是两种独立 API。上面的完整值方法会分配返回的 `String` 或 `Vec`；
transcoder 方法则写入调用方提供的缓冲区。

```rust
use qubit_codec::Transcoder;
use qubit_codec_misc::{Base64Codec, Base64Decoder};

let mut decoder = Base64Decoder::standard();
let mut raw = [0; 3];
let n = decoder.transcode_complete_into(b"TQ==", &mut raw).expect("valid Base64");
assert_eq!(&raw[..n], b"M");

let mut encoder = Base64Codec::standard().encoder();
let mut text = [0; 8];
let n = encoder.transcode_complete_into(b"M", &mut text).expect("sufficient bound");
assert_eq!(&text[..n], b"TQ==");
```

流式处理时，在第一个 chunk 前调用 `reset`。decoder 不会保存不完整的文本尾部，
所以调用方应保留未消费输入，并在新源数据到达后将其合并再试。只有数据源确认 EOF 后，
才调用 `transcode_eof`；成功处理最终输入后再调用 `finish`。

```rust
use qubit_codec::{TranscodeStatus, Transcoder};
use qubit_codec_misc::Base64Decoder;

let mut decoder = Base64Decoder::standard();
decoder.reset(&mut [], 0).expect("reset");
let mut raw = [0; 3];
let p = decoder.transcode(b"TQ", 0, &mut raw, 0).expect("open stream");
assert_eq!(p.read(), 0);
assert!(matches!(p.status(), TranscodeStatus::NeedInput { .. }));
// 保留未消费的 TQ，并拼接源数据中的末尾 ==。
let p = decoder.transcode_eof(b"TQ==", 0, &mut raw, 0).expect("final input");
assert_eq!(&raw[..p.written()], b"M");
decoder.finish(&mut [], 0).expect("close stream");
```

encoder 会在内部保留短输入，因此调用方不必在多次调用之间保存部分字节尾部；
最终部分组只会由 `finish` 输出。

```rust
use qubit_codec::Transcoder;
use qubit_codec_misc::Base64Encoder;

let mut encoder = Base64Encoder::standard();
let mut text = [0; 8];
encoder.reset(&mut text, 0).expect("reset");
assert_eq!(encoder.transcode(b"M", 0, &mut text, 0).unwrap().written(), 0);
assert_eq!(encoder.transcode(b"a", 0, &mut text, 0).unwrap().written(), 0);
assert_eq!(encoder.transcode(b"n", 0, &mut text, 0).unwrap().written(), 4);
assert_eq!(&text[..4], b"TWFu");
assert_eq!(encoder.finish(&mut text, 4).unwrap(), 0);
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

十六进制转义读取一到两位数字，八进制转义最多读取三位，结果保留低八位。`\u` 固定读取
4 位十六进制数字，`\U` 固定读取 8 位；两者的值都必须能放入一个字节，不会将 Unicode
转换为 UTF-8。输入两端的引号不会自动剥除。例如，`\x123` 得到 `[0x12, b'3']`，`\777`
得到 `[0xff]`。原始字符仅允许 printable ASCII、制表符、换行符、垂直制表符和换页符；
其他字符必须转义。
转义空格（`\ `）也可使用。处理流式输入时，确认 EOF 后调用 `Codec::decode_eof`，
以解析格式相关的短转义。

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

这些 trait 定义在 `qubit-codec` crate 中，因此泛型调用方应将该 crate
声明为直接依赖。

```rust
use qubit_codec::ValueEncoder;
use qubit_codec_misc::{
    MiscCodecError,
    HexCodec,
};

fn encode_payload<C>(
    codec: &mut C,
    payload: &[u8],
) -> Result<String, MiscCodecError>
where
    C: ValueEncoder<[u8], Output = String, Error = MiscCodecError>,
{
    codec.encode(payload)
}

fn main() {
    let mut codec = HexCodec::new();
    let text = encode_payload(&mut codec, &[0xab, 0xcd])
        .expect("hex encoding should not fail");
    assert_eq!("abcd", text);
}
```

## 安全的 Hex 字节流式 Adapter

添加以下两个直接依赖：

```toml
[dependencies]
qubit-codec = "0.17"
qubit-codec-misc = "0.8"
```

`HexByteCodec` 每次解码一个字节。它不接受 `HexCodec` 的整体前缀、逐字节前缀或分隔符配置，这些格式选项不能在两个 codec 之间通用。将未消费输入保留在 `pending`，只移除 `read` 指示的前缀；输出中只有 `written` 指示的前缀有效。本例的固定两字节输出足够，因此显式保证不会出现 `NeedOutput`；通用驱动遇到 `NeedOutput` 时必须扩容或换用更大的输出缓冲区，并用未消费输入重试。`Complete` 只表示本次调用消费了当前可见输入，并不表示源已经到达 EOF。只有上游确认 EOF 后，才对 pending 尾部调用 `transcode_eof`，随后调用 `finish`。

```rust
use qubit_codec::CodecTranscodeDecoder;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_codec_misc::HexByteCodec;

fn main() {
    let mut decoder = CodecTranscodeDecoder::new(HexByteCodec::new());
    let chunks: [&[u8]; 3] = [b"1", b"f8", b"b"];
    let mut pending = Vec::new();
    let mut output = [0_u8; 2];
    let mut written = 0;

    decoder.reset(&mut output, 0).expect("reset decoder");
    for (index, chunk) in chunks.iter().enumerate() {
        pending.extend_from_slice(chunk);
        let progress = decoder
            .transcode(&pending, 0, &mut output, written)
            .expect("decode available input");
        let read = progress.read();
        written += progress.written();
        pending.drain(..read);
        assert!(!matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
        if index < 2 {
            assert!(matches!(progress.status(), TranscodeStatus::NeedInput { .. }));
        } else {
            assert_eq!(TranscodeStatus::Complete, progress.status());
        }
    }

    // Complete says this call consumed its visible input; only the source tells us it has ended.
    assert!(pending.is_empty());
    let eof = decoder
        .transcode_eof(&pending, 0, &mut output, written)
        .expect("finish decoding at confirmed EOF");
    assert_eq!(TranscodeStatus::Complete, eof.status());
    written += eof.written();
    assert_eq!([0x1f, 0x8b], output);
    assert_eq!(2, written);
    assert_eq!(0, decoder.finish(&mut output, written).expect("finish adapter"));

    output = [0xa5; 2];
    decoder.reset(&mut output, 0).expect("reset for short input");
    let error = decoder.transcode_eof(b"1", 0, &mut output, 0).unwrap_err();
    assert!(matches!(
        error,
        TranscodeDecodeError::Failure(TranscodeFailure::IncompleteInput {
            input_index: 0,
            required: 2,
            available: 1
        })
    ));
    assert_eq!([0xa5; 2], output);

    decoder.reset(&mut output, 0).expect("reset for invalid input");
    let error = decoder.transcode(b"1g", 0, &mut output, 0).unwrap_err();
    assert!(matches!(
        error.domain_ref(),
        Some(qubit_codec_misc::MiscCodecError::InvalidDigit { index: 1, .. })
    ));
    assert_eq!([0xa5; 2], output);
}
```

关于相关的短转义边界，可参阅 [CStringLiteralCodec 的 EOF 短转义测试](https://github.com/qubit-ltd/rs-codec-misc/blob/main/tests/codec_tests.rs)；该测试检查短转义只在确认 EOF 后才按 EOF 解析。

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

### `HexCodecConfig` 操作

| 方法 | 描述 |
|------|------|
| `new()` | 创建默认的小写、连续十六进制配置 |
| `with_uppercase(enabled)` | 配置字符大小写 |
| `with_prefix(prefix)` | 配置整体前缀 |
| `with_byte_prefix(prefix)` | 配置每个 byte 前的前缀 |
| `with_separator(separator)` | 配置字节之间的分隔符 |
| `with_ignored_ascii_whitespace(enabled)` | 配置解码时的空白处理 |
| `with_ignore_prefix_case(enabled)` | 配置前缀大小写不敏感匹配 |
| `is_uppercase()` / `prefix()` / `byte_prefix()` | 查看输出格式配置 |
| `separator()` / `ignores_ascii_whitespace()` / `ignores_prefix_case()` | 查看解码策略配置 |
| `HexCodec::from_config(config)` | 使用可复用配置创建 codec |
| `HexCodec::config()` | 借用 codec 当前配置 |

### `HexByteCodec` 操作

| 方法或 Trait | 描述 |
|--------------|------|
| `new()` | 创建小写的单字节 hex codec |
| `upper()` | 创建大写的单字节 hex codec |
| `with_uppercase(enabled)` | 配置字符大小写 |
| `Codec<Value = u8, Unit = u8>` | 将一个 byte 编码或解码为正好两个 ASCII hex unit |

### `Base64Codec` 操作

| 方法 | 字母表 | Padding | 描述 |
|------|--------|---------|------|
| `standard()` | 标准 | 有 | 创建标准 Base64 codec |
| `standard_no_pad()` | 标准 | 无 | 创建无 padding 的标准 Base64 codec |
| `url_safe()` | URL-safe | 有 | 创建 URL-safe Base64 codec |
| `url_safe_no_pad()` | URL-safe | 无 | 创建无 padding 的 URL-safe Base64 codec |
| `encode(bytes)` | 已配置 | 已配置 | 将字节编码为 Base64 文本 |
| `decode(text)` | 已配置 | 已配置 | 将 Base64 文本解码为字节 |

`Base64Codec` 是完整值 owned facade，没有 `decode_into`；如需解码到调用方缓冲区，
请使用 `Base64Decoder` 和 `Transcoder`。`Base64Codec::encoder()` 与 `decoder()` 会按 facade
配置创建独立的流状态。两个方向的类型还分别提供 `standard()`、`standard_no_pad()`、
`url_safe()` 和 `url_safe_no_pad()` 构造器；`Default` 对应标准带 padding 的 Base64。

带 padding 与无 padding 模式不会互相接受对方的部分组形式。例如，
`Base64Codec::standard().decode("Zg")` 和
`Base64Codec::standard_no_pad().decode("Zg==")` 都会返回错误；完整三字节组
`"TWFu"` 不需要 padding，两种模式均可接受。

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
| `InvalidBase64` | Base64 解码失败，携带结构化错误分类并保留底层 `base64::DecodeError` |
| `InvalidUtf8` | 解码后的字节不是合法 UTF-8 |

完整值辅助方法会将截断的 `%XX` 和 C 转义报告为 `InvalidEscape`。低层
`Codec::decode` 在开放流尾部使用 `qubit_codec::DecodeFailure::Incomplete`；
调用方确认 EOF 后应使用 `Codec::decode_eof`，以便解析格式相关的短转义。

## 性能考虑

Codec 实现直接操作借用的 byte slice 或字符串，只在目标格式确实需要时返回 owned output。
配置存放在小型值类型中，泛型 trait 用法也不要求动态分发。

可运行 `cargo bench --bench codec_benchmarks -- base64-transcoders` 测量 64 B 和 64 KiB
输入下的 Base64 owned、调用方缓冲区和流式路径。流式用例包含 7 字节与 4096 字节分片；
模式抽样也覆盖其他字母表及 padding 配置。输出缓冲区在计时循环前分配。owned 用例包含输出分配，
因此其耗时不能直接视为 codec 本身的处理耗时。基准运行时会报告实测吞吐，本文不预设吞吐数值。
`fuzz/` 下的 target 可用 `cargo +nightly fuzz build` 编译，并用
`cargo +nightly fuzz run percent_and_form`（或其他 target 名称）运行。

## 依赖项

运行时依赖保持很少：

- `base64` 提供 Base64 engine。
- `percent-encoding` 提供百分号转义字节表。
- `thiserror` 提供公共错误类型实现。

`form_urlencoded` 仅用于差分测试和 fuzz target，不进入运行时 API。

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./.infra/bin/ci-check.sh

# 检查代码覆盖率
./.infra/bin/coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./.infra/bin/align-ci.sh`格式化代码，运行`./.infra/bin/ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-codec-misc](https://github.com/qubit-ltd/rs-codec-misc)
