"""headless shell 测试"""
from playwright.sync_api import sync_playwright
exec_path = '/Users/xujian/Library/Caches/ms-playwright/chromium_headless_shell-1217/chrome-headless-shell-mac-arm64/chrome-headless-shell'
with sync_playwright() as p:
    b = p.chromium.launch(executable_path=exec_path, headless=True, args=['--no-sandbox'])
    print('HEADLESS_SHELL OK')
    b.close()
