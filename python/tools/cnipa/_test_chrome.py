"""测试系统 Google Chrome + Playwright 能否访问 CNIPA"""
from playwright.sync_api import sync_playwright
with sync_playwright() as p:
    b = p.chromium.launch(channel='chrome', headless=True, args=['--no-sandbox','--disable-blink-features=AutomationControlled'])
    ctx = b.new_context(user_agent='Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36', locale='zh-CN', viewport={'width':1280,'height':900})
    page = ctx.new_page()
    page.goto('http://epub.cnipa.gov.cn/', wait_until='load', timeout=60000)
    print('Page title:', page.title())
    print('Has searchStr:', page.query_selector('#searchStr') is not None)
    ctx.close()
    b.close()
    print('OK')
