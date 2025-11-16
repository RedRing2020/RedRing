// RedRing Documentation Custom JavaScript

document.addEventListener("DOMContentLoaded", function () {
  // テスト用：CSSが効いていることを確認するための明確な変更
  console.log("RedRing Custom JS Loaded!");
  
  // テスト用：ページタイトルに装飾を追加
  const title = document.querySelector('.menu-title');
  if (title) {
    title.innerHTML = '🦀 ' + title.innerHTML + ' 🔧';
  }
  // ステータスバッジの自動追加
  const badges = {
    "✅ 実装完了": "status-completed",
    "🚧 実装中": "status-wip",
    "📋 計画中": "status-planned",
  };

  // h3要素にバッジを自動追加
  document.querySelectorAll("h3, h4").forEach((heading) => {
    Object.keys(badges).forEach((status) => {
      if (heading.textContent.includes(status)) {
        const badge = document.createElement("span");
        badge.className = `status-badge ${badges[status]}`;
        badge.textContent = status;
        heading.appendChild(badge);
      }
    });
  });

  // コードブロックに言語ラベル追加
  document.querySelectorAll('pre code[class*="language-"]').forEach((code) => {
    const lang = code.className.match(/language-(\w+)/)?.[1];
    if (lang) {
      const label = document.createElement("div");
      label.className = "code-lang-label";
      label.textContent = lang.toUpperCase();
      label.style.cssText = `
                position: absolute;
                top: 0.5rem;
                right: 0.5rem;
                background: var(--primary-color);
                color: white;
                padding: 0.25rem 0.5rem;
                border-radius: 0.25rem;
                font-size: 0.75rem;
                font-weight: 600;
            `;
      code.parentElement.style.position = "relative";
      code.parentElement.appendChild(label);
    }
  });

  // 目次の自動生成
  function generateTOC() {
    const headings = document.querySelectorAll("h2, h3");
    if (headings.length === 0) return;

    const toc = document.createElement("div");
    toc.className = "auto-toc";
    toc.innerHTML = "<h3>📋 目次 / Table of Contents</h3>";

    const list = document.createElement("ul");
    list.style.cssText = `
            background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
            padding: 1rem;
            border-radius: 8px;
            margin: 1rem 0;
            border-left: 4px solid var(--primary-color);
        `;

    headings.forEach((heading, index) => {
      const li = document.createElement("li");
      const link = document.createElement("a");

      // IDを生成
      const id = `toc-${index}`;
      heading.id = id;

      link.href = `#${id}`;
      link.textContent = heading.textContent;
      link.style.cssText = `
                color: var(--primary-color);
                text-decoration: none;
                font-weight: ${heading.tagName === "H2" ? "600" : "400"};
                margin-left: ${heading.tagName === "H3" ? "1rem" : "0"};
            `;

      li.appendChild(link);
      list.appendChild(li);
    });

    toc.appendChild(list);

    // 最初のh2の前に挿入
    const firstH2 = document.querySelector("h2");
    if (firstH2) {
      firstH2.parentNode.insertBefore(toc, firstH2);
    }
  }

  // 長いページの場合のみ目次生成
  if (document.querySelectorAll("h2").length >= 3) {
    generateTOC();
  }

  // スムーススクロール
  document.querySelectorAll('a[href^="#"]').forEach((anchor) => {
    anchor.addEventListener("click", function (e) {
      e.preventDefault();
      const target = document.querySelector(this.getAttribute("href"));
      if (target) {
        target.scrollIntoView({
          behavior: "smooth",
          block: "start",
        });
      }
    });
  });

  // コードブロックのコピー機能
  document.querySelectorAll("pre").forEach((pre) => {
    const button = document.createElement("button");
    button.textContent = "📋";
    button.title = "Copy code";
    button.style.cssText = `
            position: absolute;
            top: 0.5rem;
            left: 0.5rem;
            background: var(--secondary-color);
            color: white;
            border: none;
            padding: 0.25rem 0.5rem;
            border-radius: 0.25rem;
            cursor: pointer;
            font-size: 0.875rem;
        `;

    button.addEventListener("click", async () => {
      const code = pre.querySelector("code");
      if (code) {
        try {
          await navigator.clipboard.writeText(code.textContent);
          button.textContent = "✅";
          setTimeout(() => {
            button.textContent = "📋";
          }, 2000);
        } catch (err) {
          console.error("Failed to copy code:", err);
        }
      }
    });

    pre.style.position = "relative";
    pre.appendChild(button);
  });

  // ページロード時のフェードイン効果
  document.body.style.opacity = "0";
  setTimeout(() => {
    document.body.style.transition = "opacity 0.5s ease";
    document.body.style.opacity = "1";
  }, 100);
});

// 数学記法サポート（MathJax）
window.MathJax = {
  tex: {
    inlineMath: [
      ["$", "$"],
      ["\\(", "\\)"],
    ],
    displayMath: [
      ["$$", "$$"],
      ["\\[", "\\]"],
    ],
  },
  svg: {
    fontCache: "global",
  },
};
