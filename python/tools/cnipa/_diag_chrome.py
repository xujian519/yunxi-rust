"""最小化测试：不同 Chromium 启动参数在 ARM Mac 上的表现"""
import sys, os
os.environ["PLAYWRIGHT_HEADED"] = "0"

from playwright.sync_api import sync_playwright

configs = [
    # (描述, kwargs)
    ("bare minimum", {"headless": True}),
    ("no-sandbox only", {"headless": True, "args": ["--no-sandbox"]}),
    ("channel chrome minimal", {"channel": "chrome", "headless": True}),
    ("disable gpu", {"headless": True, "args": ["--no-sandbox", "--disable-gpu"]}),
    ("old headless", {"headless": True, "args": ["--no-sandbox", "--headless=old"]}),
]

for desc, kwargs in configs:
    try:
        with sync_playwright() as p:
            b = p.chromium.launch(**kwargs)
            b.close()
            print(f"✅ {desc}: OK")
    except Exception as e:
        print(f"❌ {desc}: {str(e)[:120]}")
