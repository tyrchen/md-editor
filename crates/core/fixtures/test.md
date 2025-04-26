# 副驾还是主驾？一次用 AI 写代码的深度体验

最近，我一直在探索 AI 编程的边界。

之前验证过的一个方向，是让 AI 仅凭一张 app 的截图，推断出其中的数据结构（schema），生成相应的 JSON，然后用 React 和 Tailwind 构建出能消费这个 JSON 的组件，最终组织成一个完整的前端应用。我已经用这个方法做了几个 app，验证下来，确实可行。原本以为只是灵光一现，结果却意外地顺畅自然，仿佛本就该如此。

另一个方向，是提供足够的上下文信息，清晰地描述需求，再补充几段相关依赖库的使用示例，让 AI 据此生成所需的功能。我用这种方式构建了不少项目，其中包括 [`tyrchen/postgres-mcp`](https://github.com/tyrchen/postgres-mcp)，一个围绕 PostgreSQL 的 mcp，效果也不错。

那如果问题再往前一步：让 AI 帮我们实现一个并不熟悉的算法，或者把某种语言中已有的实现迁移到另一种语言，比如从 C 转到 Rust，它还能胜任吗？

带着这个问题，昨晚我开始尝试用 Cursor 从零实现一个 diff / patch 的功能，目标语言是 Rust。从最初的构思，到一步步的提示和反馈，仿佛是在和另一个程序员结伴夜谈，只不过它不会困，也不会倦。

## 为什么选择 diff / patch？

diff / patch 算法本身相对成熟，属于 AI 能够“理解”的那一类问题。在 Rust 的生态里，有一个叫 `similar` 的热门 diff 库，而 Git 自带的 `xdiff` 也是一个功能完整的 C 实现。从 AI 的角度看，这类题目一方面不缺训练语料，算法也有清晰的结构，不至于陷入“你说我猜”的尴尬；另一方面，如果 AI 不能立刻给出可用的版本，我还可以引导它“参考”已有实现，以此修正偏差。

换句话说，选它，是因为既能考验 AI 的构造能力，又留有补救的余地，进可攻，退可守。

## 起手式：定义基本功能

要让 AI 写出靠谱的代码，一份清晰、具体的 spec 几乎是不可或缺的前提。我在第一版 spec 中，先定义了几个基础的数据结构，它们既是我对 diff / patch 问题的初步理解，也是在和 AI 对话前写下的“注脚”。

这些结构，一方面是引导 AI 理解任务边界的锚点，另一方面，也是我自己思考的起点：通过把脑海中模糊的算法轮廓，压缩进类型定义和字段设计，我试图把一个抽象的问题，转化为一组可讨论、可生成的具体约束。

这一步做得是否扎实，往往决定了 AI 后续的输出是清晰聚焦，还是漫无边际。

```rust
struct Differ {
  old: String,
  new: String,
  ...
}

struct Patcher {
  patch: Patch,
  ...
}

struct Patch {
    ...
}

impl Differ {
  pub fn generate(&self) -> Patch {
    // ...
  }
}

impl Patcher {
  pub fn apply(&self, content: &str, reverse: bool) -> Result<String, Error> {
    // ...
  }
}
```

第一版的代码是由 claude 3.7 完成的，质量不错，也有基础的单元测试。

![image.png](%E5%89%AF%E9%A9%BE%E8%BF%98%E6%98%AF%E4%B8%BB%E9%A9%BE%EF%BC%9F%E4%B8%80%E6%AC%A1%E7%94%A8%20AI%20%E5%86%99%E4%BB%A3%E7%A0%81%E7%9A%84%E6%B7%B1%E5%BA%A6%E4%BD%93%E9%AA%8C%201a42edc7ec6d801eaecedafd75574651/image.png)

## 进阶：多文件 patch

当基础的 diff / patch 功能通过了最初级的测试之后，我很快开始尝试推进下一步——支持多文件的 patch。

这种迫不及待，其实是出于一种惯常的开发节奏：一旦看到某个路径是通的，就想看看它能走多远、扛多重。我随即补充了一个新的 spec，里面加入了多文件处理所需的数据结构，为 AI 描绘了进一步的目标边界。

在这一步，我没有推倒重来，而是在原有结构的基础上做了增量修改，既保留了之前实现的上下文，也让 AI 能够延续已有的理解，继续前行。

```rust
struct MultifilePatch {
  patches: Vec<Patch>,
  ...
}

struct MultifilePatcher {
  patches: Vec<Patch>,
  ...
}

struct PatchedFile {
  path: String,
  content: String,
}

impl MultifilePatcher {
  pub fn apply(&self, reverse: bool) -> Result<Vec<PatchedFile>, Error> {
    // ...
  }
}
```

第二版的代码依然由 Claude 3.7 完成，虽经历了一些波折，最终还是通过了单元测试。之后，我又让 AI 添加了集成测试，结果也顺利通过，像是走出迷宫后的那口长气。

## 重构：添加更多 differ

最初的代码完成后，`src/` 目录下有三个文件：`lib.rs`、`differ.rs` 和 `patcher.rs`。接下来，我打算引入 Myers 算法，于是添加了一个新的 spec，告诉 AI 我希望将 `differ.rs` 的逻辑重构为以下目录结构：

```bash
differ/
├── mod.rs
├── myers.rs
├── naive.rs
└── README.md
```

旧的逻辑迁移到 `naive.rs`，新的 Myers 算法则放入 `myers.rs`。因为现在有多个实现方式，我补充了一条说明：将原来 `pub fn generate(&self) -> Patch` 抽象为一个 trait，让两种算法都实现它。

这次重构本身很顺利，Claude 3.7 轻松完成。但当它开始实现 Myers 算法时，却显得力不从心。它不断耗光上下文，还总用尽 25 次调用，却始终没能产出能通过测试的版本。我尝试换 prompt，调整结构，仍是徒劳。

最终它“崩溃”了：在某次尝试中，它直接删光自己写的数百行代码，然后把 `naive.rs` 的逻辑稍作修改拷贝过来，以求测试通过。我一度惊讶：“咦？前一版还有六个测试没过，下一版居然都绿了？”

这下可算真正明白了：AI 被逼急了，行径和人类也没太大差别——投机取巧那点小聪明，它也会。

最后我实在无计可施，换用了 Google 的 Gemini 2.5 Pro，让它从头写一版 `myers.rs`。与 Claude 不同，Gemini 带有“思考”——它一上来就开始自顾自碎碎念，几十秒内默默演算、规划、下笔，一气呵成地写出一个几乎能跑通的版本。之后我们又一起迭代了两三轮，Myers 算法总算落地了。

![image.png](%E5%89%AF%E9%A9%BE%E8%BF%98%E6%98%AF%E4%B8%BB%E9%A9%BE%EF%BC%9F%E4%B8%80%E6%AC%A1%E7%94%A8%20AI%20%E5%86%99%E4%BB%A3%E7%A0%81%E7%9A%84%E6%B7%B1%E5%BA%A6%E4%BD%93%E9%AA%8C%201a42edc7ec6d801eaecedafd75574651/image%201.png)

添加了新的算法后，原有代码中重复逻辑变得不可避免。虽然我在 Cursor 的 rules 里设了专门的 `cleancode.mdc` 来强调 DRY（Don't Repeat Yourself），但眼下的重复已堆积得相当可观。

于是我让 Gemini 单独重构了一些文件，目标是：精简代码（DRY）、清晰拆分逻辑（SRP）、顺带进行一些性能优化。它将共用部分提取到 `differ/common.rs`，在保持测试全绿的同时，也让结构更加工整清晰。

## 性能：对比 naive / myers 算法

完成两种算法之后，我开始关心它们的性能差异。为此，我写了一个新的 spec，请 AI 使用 `divan` 来构建性能测试代码。

我本人没用过 `divan`，看起来它和 `criterion` 很像，但语法略为轻量。我便在 spec 中提供了官方文档里的两个示例：一个简洁，一个复杂，供 AI 参考。这样做的目的，是让它既能快速入门，又有足够的上下文生成符合我们需求的测试逻辑。

```rust
fn main() {
    // Run registered benchmarks.
    divan::main();
}

// Register a `fibonacci` function and benchmark it over multiple cases.
#[divan::bench(args = [1, 2, 4, 8, 16, 32])]
fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        1
    } else {
        fibonacci(n - 2) + fibonacci(n - 1)
    }
}
```

大模型在 few-shot learning 上的表现确实惊人。我常常在 spec 里用这种方式“喂”给模型它可能没学过、或者掌握得不够牢靠的内容示例。只要示例写得合适，它便能迅速依葫芦画瓢，举一反三。

性能测试的代码也是这么搞出来的。经过几轮迭代，很快就能稳定运行了。但让我没想到的是：在跑 benchmark 时，一个小插曲出现了：AI 手搓的贪心算法（naive 实现），在 diff 效率上，居然比 Myers 算法高了好几倍。

这结果让我颇为意外。于是我把测试结果丢给 Gemini，请它重新审视两个实现的结构，思考性能差异可能来自哪里，并请它写一篇简洁的对比分析。对于这个问题，Gemini 是这么说的：

```markdown
XDiffDiffer Slower Due To:

1. Complex algorithm design for optimal diff finding
2. Significant preprocessing overhead
3. Higher memory requirements
4. Complex path tracking calculations
5. Recursive implementation

NaiveDiffer Faster Due To:

1. Simple greedy approach
2. Limited lookahead strategy
3. Minimal computational overhead
4. Linear progression through files
```

后来我让 Gemini 对 Myers 算法做了几轮优化，最终的性能表现仍然和最初版本相差无几。

## 转写：让 AI 把 C 代码转为 Rust

对于前面测出来的性能差异，我始终将信将疑。于是，我决定另辟蹊径，把 Git 源码中的 `xdiff` 目录整体复制到我的代码库中，然后点名让 AI 阅读 `xdiffi.c`，并按照我现有的数据结构和 trait 系统，写出一个 Rust 版本的 xdiff 算法。

这个过程并不轻松。经历了若干轮试错之后，Gemini 终于完成了一个能通过测试的版本。为了更系统地做性能对比，我还让它顺手实现了一个基于 `similar` crate 的封装（命名为 `SimilarDiffer`），并为这四种实现都构建了 benchmark 测试。

对一段大约 100 行的代码差异（diff）进行测试后，四种算法的 median 时间如下：

- Naive 贪婪算法：**4.433 µs**
- Gemini 实现的 Myers 算法：**11.44 µs**
- Gemini 参考 C 实现的 xdiff 算法：**9.483 µs**
- `similar` 默认算法：**22.73 µs**

这个结果多少有些出人意料。看起来在变更量较小的场景下，结构简单、决策激进的贪婪算法反而表现更好。也提醒我一个老道理：算法优劣，常常要结合使用场景来看。

在这个过程中，把 C 代码一比一翻译成 Rust 这件事，其实并不算难。真正有挑战的是：如何在既定的数据结构、方法签名、trait 系统下，用 idiomatic Rust 重构出等价功能的实现。这一点上，AI 展现出了惊人的优势。原本约 1000 行的 C 代码，转换后是大约 700 行的 Rust。我一开始担心转换不完全，于是让 Gemini 仔细对比功能点，补齐边角，并添加了大量测试用例。测试通过后，我这才放心。

## 文档：AI 的伟大舞台

让 AI 写 README.md 早已不新鲜。但很多时候，README 只能回答“这是什么”，却难以清晰地说明“这怎么用”，更别说“怎么用得好”。

这正是 AI 大有可为的地方。得益于越来越强大的大语言模型（比如 Gemini 2.5），现在我们不仅能让它写出结构清晰、语句通顺的文档，还能让它自动生成可运行的、循序渐进的教学代码，甚至生成多语言版本的教程。

我使用 [The-Pocket/Tutorial-Codebase-Knowledge](https://github.com/The-Pocket/Tutorial-Codebase-Knowledge) 项目为当前这个 crate 生成了中英文两个版本的完整教程。简单浏览了一下，讲解思路清晰、代码段插得恰到好处，看起来已经足够支撑初学者快速入门。

![image.png](%E5%89%AF%E9%A9%BE%E8%BF%98%E6%98%AF%E4%B8%BB%E9%A9%BE%EF%BC%9F%E4%B8%80%E6%AC%A1%E7%94%A8%20AI%20%E5%86%99%E4%BB%A3%E7%A0%81%E7%9A%84%E6%B7%B1%E5%BA%A6%E4%BD%93%E9%AA%8C%201a42edc7ec6d801eaecedafd75574651/image%202.png)

内容娓娓道来，并不生硬：

![image.png](%E5%89%AF%E9%A9%BE%E8%BF%98%E6%98%AF%E4%B8%BB%E9%A9%BE%EF%BC%9F%E4%B8%80%E6%AC%A1%E7%94%A8%20AI%20%E5%86%99%E4%BB%A3%E7%A0%81%E7%9A%84%E6%B7%B1%E5%BA%A6%E4%BD%93%E9%AA%8C%201a42edc7ec6d801eaecedafd75574651/image%203.png)

自动生成各种图表，帮助理解内部实现：

![image.png](%E5%89%AF%E9%A9%BE%E8%BF%98%E6%98%AF%E4%B8%BB%E9%A9%BE%EF%BC%9F%E4%B8%80%E6%AC%A1%E7%94%A8%20AI%20%E5%86%99%E4%BB%A3%E7%A0%81%E7%9A%84%E6%B7%B1%E5%BA%A6%E4%BD%93%E9%AA%8C%201a42edc7ec6d801eaecedafd75574651/image%204.png)

有趣的是，为我生成文档和教程的工具本身……也是 AI 生成的。这有点像编译器的自举（bootstrap）：AI 帮我写文档，而写这个文档的 AI 工具，也是 AI 写的。想到这里，不禁莞尔。

## 尾声：不到 24 小时，我完成了 6000 行 Rust 代码

在构建整个 diff/patch 系统的过程中，作为一个人类，一个项目的发起者，我真正扮演的角色其实非常明确：

- 撰写明确的 spec
- 指挥 AI 根据 spec 工作
- 一旦陷入死循环或长时间无果，就果断换模型、重启任务，必要时推倒重来

所以我并不需要 996，也无需关灯熬夜。白天照常上班、开会，晚上陪孩子，甚至是一日三餐的间隙，和 AI 交谈几句，它就继续默默工作。而最终的成果：6000 行 Rust 代码，96 个单元测试，十余轮重构，以及成体系的文档和中英文教程，就这样静悄悄地完成了。这在半年前，我是连想都不敢想的。

现在的 AI 编程体验，很像是处于 L2 阶段的自动驾驶：你给出方向（spec），平时让系统自动行驶，只有在复杂路段或偏离轨道时，才需要接管。你不再需要关注每一行代码，而是开始关注结构、目标和约束。只要你脑中能维持一套清晰的模型，甚至可以在多个项目之间游刃有余地切换，像一个指挥多轨协奏的作曲家。

AI 编程，正在悄悄改变我们的创造方式。而我们，也许正站在一个全新的工程范式的起点上。

本文涉及的代码库：[https://github.com/tyrchen/patcher](https://github.com/tyrchen/patcher)
