"""终极诊断：crashpad 标志 + WebKit"""
from playwright.sync_api import sync_playwright

results = []

# 1. Chromium + disable crashpad
try:
    with sync_playwright() as p:
        b = p.chromium.launch(headless=True, args=['--no-sandbox', '--disable-crashpad-for-testing'])
        b.close()
        results.append("✅ chromium + disable-crashpad-for-testing")
except Exception as e:
    results.append(f"❌ chromium + crashpad flag: {str(e)[:100]}")

# 2. Firefox
try:
    with sync_playwright() as p:
        b = p.firefox.launch(headless=True)
        b.close()
        results.append("✅ firefox")
except Exception as e:
    results.append(f"❌ firefox: {str(e)[:100]}")

# 3. WebKit
try:
    with sync_playwright() as p:
        b = p.webkit.launch(headless=True)
        b.close()
        results.append("✅ webkit")
except Exception as e:
    results.append(f"❌ webkit: {str(e)[:100]}")

for r in results:
    print(r)
