"""Rosetta x86_64 测试"""
from playwright.sync_api import sync_playwright
# 使用 x86_64 Chromium
exec_path = '/Users/xujian/Library/Caches/ms-playwright/chromium-1217/chrome-mac/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing'
import os
print('path exists:', os.path.exists(exec_path))
try:
    with sync_playwright() as p:
        b = p.chromium.launch(executable_path=exec_path, headless=True, args=['--no-sandbox'])
        print('x86_64 CHROMIUM OK')
        b.close()
except Exception as e:
    print(f'FAIL: {str(e)[:200]}')
