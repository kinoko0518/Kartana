# aozora_parser

青空文庫形式のテキストをパースし、EPUB/XHTMLを生成するRustライブラリです。

## 使い方

### シンプルな使い方（高レベルAPI）

```rust
use aozora_parser::{text_to_xhtml, text_to_epub};

// XHTMLに変換
let output = text_to_xhtml(aozora_text)?;
println!("タイトル: {}", output.metadata.title);
println!("目次数: {}", output.toc.len());

// EPUBファイルを直接生成
text_to_epub(aozora_text, "output.epub")?;
```

### 詳細な制御（低レベルAPI）

```rust
use aozora_parser::{parse_aozora, parse, parse_blocks, EpubGenerator, Span};

// 1. トークン化
let tokens = parse_aozora(text)?;

// 2. パース（メタデータ抽出 + ルビ処理）
let doc = parse(tokens)?;
println!("タイトル: {}", doc.metadata.title);
println!("著者: {}", doc.metadata.author);

// 3. ブロック構造解析
let blocks = parse_blocks(doc.items)?;

// 4. 生成
let generator = EpubGenerator::new(doc.metadata.title, doc.metadata.author, blocks);
generator.write_to_file("output.epub")?;
```

### Span（位置情報）の活用

各トークンとパース結果は元テキストの位置情報を持ちます：

```rust
use aozora_parser::{parse_aozora, AozoraToken, Span};

let tokens = parse_aozora("漢字《かんじ》".to_string())?;

if let AozoraToken::Text(t) = &tokens[0] {
    println!("'{}' は {}〜{} 文字目", t.content, t.span.start, t.span.end);
    // => '漢字' は 0〜2 文字目
}
```

---

## アーキテクチャ

### 処理パイプライン

```
テキスト → Tokenizer → Parser → BlockParser → Linter → Generator → EPUB/XHTML
            ↓           ↓          ↓            ↓
        AozoraToken  ParsedItem  AozoraBlock  LintResult
```

### Linter（検証）

表記規則の検証と警告を提供します：

```rust
use aozora_parser::{text_to_xhtml_with_lint, Severity};

let (xhtml, toc, metadata, warnings) = text_to_xhtml_with_lint(aozora_text)?;

for w in &warnings {
    match w.severity {
        Severity::Error => eprintln!("エラー: {}", w.message),
        Severity::Warning => eprintln!("警告: {}", w.message),
        Severity::Info => eprintln!("情報: {}", w.message),
    }
}
```

#### 検出される警告

| 種類 | 説明 |
|------|------|
| `MissingParagraphIndent` | 段落先頭に字下げがない |
| `PunctuationBeforeQuote` | `。」`パターン（`」。`が推奨） |
| `OddEllipsisCount` | `…`/`―`が奇数個（偶数個が推奨） |
| `InvalidCharAfterExclamation` | `！？`の後に空白/括弧がない |

---

## 抽象レイヤ（概念・設計）

### 1. トークン化層

**責務**: 生テキストを意味のある単位（トークン）に分割

| 概念 | 説明 |
|------|------|
| TextToken | テキストの断片（漢字/ひらがな/カタカナ/その他を区別） |
| Ruby | ルビ（振り仮名）`《...》` |
| RubySeparator | ルビ範囲指定子 `｜` |
| Command | 注記コマンド `［＃...］` |
| Span | 元テキスト内での位置情報 |

### 2. パース層

**責務**: トークン列を構造化されたドキュメントに変換

| 概念 | 説明 |
|------|------|
| AozoraDocument | パース済みドキュメント全体 |
| AozoraMetadata | タイトル・著者情報 |
| DecoratedText | テキスト + オプションのルビ |
| ParsedItem | パース済み要素（Text/Command/Newline/SpecialCharacter） |

### 3. ブロック構造層

**責務**: フラットなアイテム列をネストしたブロック構造に変換

| 概念 | 説明 |
|------|------|
| AozoraBlock | ブロック要素（装飾 + 子要素） |
| BlockElement | ブロック内の要素（Item or 入れ子Block） |
| CommandBegin/End | 見出し、字下げ等のブロック開始/終了 |

### 4. 生成層

**責務**: ブロック構造からXHTML/EPUBを生成

| 概念 | 説明 |
|------|------|
| XhtmlGenerator | XHTML生成器 |
| EpubGenerator | EPUB生成器 |
| TocEntry | 目次エントリ |

---

## 具体レイヤ（ファイル・型）

### ディレクトリ構成

```
src/
├── lib.rs              # 公開API・高レベル関数
├── tokenizer.rs        # トークナイザー本体
├── tokenizer/
│   └── command.rs      # コマンドパーサー
├── parser.rs           # パーサー本体
├── parser/
│   └── tests.rs        # パーサーテスト
├── block_parser.rs     # ブロック構造解析
├── linter.rs           # 検証・警告
├── xhtml_generator.rs  # XHTML生成
├── epub_generator.rs   # EPUB生成
├── css.rs              # デフォルトCSS
└── epub_template/      # EPUBテンプレートファイル
```

### 主要な型

#### トークン層 (`tokenizer.rs`)

```rust
pub struct Span { pub start: usize, pub end: usize }
pub struct TextToken { pub content: String, pub kind: TextKind, pub span: Span }
pub struct CommandToken { pub content: String, pub span: Span }

pub enum AozoraToken {
    Text(TextToken),
    Ruby { content: String, span: Span },
    RubySeparator(Span),
    Command(CommandToken),
    Newline(Span),
    Odoriji(Span),
    DakutenOdoriji(Span),
}
```

#### パース層 (`parser.rs`)

```rust
pub struct DecoratedText { pub text: String, pub ruby: Option<String>, pub span: Span }

pub enum ParsedItem {
    Text(DecoratedText),
    Command { cmd: Command, span: Span },
    Newline(Span),
    SpecialCharacter { kind: SpecialCharacter, span: Span },
}
```

#### ブロック層 (`block_parser.rs`)

```rust
pub struct AozoraBlock {
    pub decoration: Option<CommandBegin>,
    pub elements: Vec<BlockElement>,
    pub span: Span,
}

pub enum BlockElement {
    Item(ParsedItem),
    Block(AozoraBlock),
}
```

### エラー型

すべてのエラー型はSpan情報を含み、エラー発生位置を特定できます：

```rust
pub enum TokenizeError {
    UnclosedCommand(Span),  // 閉じられていないコマンド
}

pub enum ParseError {
    UnexpectedToken { token: AozoraToken, span: Span },
}

pub enum BlockParseError {
    UnexpectedEnd { end: CommandEnd, span: Span },
    UnclosedBlock { begin: CommandBegin, span: Span },
}
```

---

## 対応する青空文庫記法

| 記法 | 例 | 説明 |
|------|-----|------|
| ルビ | `漢字《かんじ》` | 直前の漢字にルビ |
| ルビ範囲指定 | `｜青空文庫《あおぞらぶんこ》` | 指定範囲にルビ |
| 見出し | `［＃大見出し］...［＃大見出し終わり］` | 見出しブロック |
| 字下げ | `［＃３字下げ］` | インデント |
| 傍点 | `［＃「...」に傍点］` | 強調 |
| 踊り字 | `／＼` `／″＼` | 繰り返し記号 |
| 改ページ | `［＃改ページ］` | ページ区切り |

---

## ライセンス

MIT
