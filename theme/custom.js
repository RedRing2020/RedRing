// RedRing Documentation Custom JavaScript

document.addEventListener('DOMContentLoaded', function() {
    // コピーボタンの機能追加
    addCopyButtons();
    
    // 数式レンダリング設定
    configureMathJax();
    
    // スムーズスクロール
    enableSmoothScrolling();
    
    // 動的テーマ調整
    adjustTheme();
});

function addCopyButtons() {
    const codeBlocks = document.querySelectorAll('pre code');
    
    codeBlocks.forEach(function(block) {
        const pre = block.parentNode;
        const button = document.createElement('button');
        button.className = 'copy-button';
        button.textContent = 'Copy';
        button.style.cssText = `
            position: absolute;
            top: 8px;
            right: 8px;
            padding: 4px 8px;
            background: rgba(59, 130, 246, 0.8);
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
            opacity: 0;
            transition: opacity 0.3s ease;
        `;
        
        pre.style.position = 'relative';
        pre.appendChild(button);
        
        // ホバーでボタン表示
        pre.addEventListener('mouseenter', function() {
            button.style.opacity = '1';
        });
        
        pre.addEventListener('mouseleave', function() {
            button.style.opacity = '0';
        });
        
        // コピー機能
        button.addEventListener('click', function() {
            navigator.clipboard.writeText(block.textContent).then(function() {
                button.textContent = 'Copied!';
                button.style.background = 'rgba(34, 197, 94, 0.8)';
                
                setTimeout(function() {
                    button.textContent = 'Copy';
                    button.style.background = 'rgba(59, 130, 246, 0.8)';
                }, 2000);
            });
        });
    });
}

function configureMathJax() {
    if (window.MathJax) {
        window.MathJax.Hub.Config({
            tex2jax: {
                inlineMath: [['$', '$']],
                displayMath: [['$$', '$$']],
                processEscapes: true
            },
            TeX: {
                Macros: {
                    // NURBS関連の数学記号
                    Vec: ["\\mathbf{#1}", 1],
                    Pt: ["\\mathbf{#1}", 1],
                    Basis: ["N_{#1,#2}", 2],
                    Rational: ["R_{#1,#2}", 2]
                }
            }
        });
    }
}

function enableSmoothScrolling() {
    const links = document.querySelectorAll('a[href^="#"]');
    
    links.forEach(function(link) {
        link.addEventListener('click', function(e) {
            e.preventDefault();
            
            const targetId = this.getAttribute('href').substring(1);
            const targetElement = document.getElementById(targetId);
            
            if (targetElement) {
                targetElement.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });
}

function adjustTheme() {
    // ダークモード対応
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    
    if (prefersDark) {
        document.body.classList.add('dark-theme');
    }
    
    // テーマ切り替え監視
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function(e) {
        if (e.matches) {
            document.body.classList.add('dark-theme');
        } else {
            document.body.classList.remove('dark-theme');
        }
    });
}

// パフォーマンス監視
if (window.performance && console.log) {
    window.addEventListener('load', function() {
        const loadTime = performance.timing.loadEventEnd - performance.timing.navigationStart;
        console.log('RedRing Documentation loaded in', loadTime, 'ms');
    });
}