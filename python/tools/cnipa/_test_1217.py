"""测试 1217 版 Chromium"""
from playwright.sync_api import sync_playwright
exec_path = '/Users/xujian/Library/Caches/ms-playwright/chromium-1217/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing'
with sync_playwright() as p:
    b = p.chromium.launch(executable_path=exec_path, headless=True, args=['--no-sandbox'])
    print('1217 LAUNCH OK')
    b.close()
