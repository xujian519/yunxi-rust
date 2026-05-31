"""Rosetta x86_64 Playwright 测试"""
import platform, subprocess, sys
print('Python arch:', platform.machine())
# 尝试用 Rosetta 运行 x86 Python
result = subprocess.run(['arch', '-x86_64', 'python3', '-c', 'import platform; print(platform.machine())'], capture_output=True, text=True)
print('Rosetta Python arch:', result.stdout.strip(), 'stderr:', result.stderr[:200])
