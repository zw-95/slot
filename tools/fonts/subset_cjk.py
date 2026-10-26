#!/usr/bin/env python3
"""裁剪中文字体，产出 slot 内置用的 label_cjk.ttf。

用法:
    py tools/fonts/subset_cjk.py <源字体.ttf> <输出.ttf>
    py tools/fonts/subset_cjk.py D:/Document/字体/霞鹜系列/LXGW975YuanSC-500W.ttf crates/slot-ui/assets/label_cjk.ttf

覆盖范围（够 UI 文案 + 绝大多数游戏名，避免整包 15 MB 进二进制）:
    ASCII / Latin-1 / LatinExt-A（拼音声调 ā á ǎ à ū ǖ 等）
    常用标点、CJK 标点、全角半角、货币与单位符号、箭头、几何与杂项符号
    日文假名、注音符号
    GB2312 一级 + 二级汉字（6763 字）
需要 fonttools:  py -m pip install fonttools brotli
"""

import subprocess
import sys
import tempfile
from pathlib import Path

# 汉字：GB2312 一二级
def gb2312_chars():
    out = []
    for hi in range(0xB0, 0xF8):
        for lo in range(0xA1, 0xFF):
            try:
                out.append(bytes([hi, lo]).decode("gb2312"))
            except UnicodeDecodeError:
                continue
    return out


def wanted():
    points = set()
    ranges = [
        (0x0020, 0x007E),  # ASCII
        (0x00A0, 0x00FF),  # Latin-1
        (0x0100, 0x024F),  # LatinExt-A/B：拼音声调 ā á ǎ à ǖ ǚ 等
        (0x2000, 0x206F),  # 通用标点 … “ ” ‘ ’ — – 等
        (0x2100, 0x214F),  # ℃ ℉ № 等
        (0x2190, 0x21FF),  # 箭头
        (0x2500, 0x25FF),  # 制表符 + 几何图形
        (0x2600, 0x26FF),  # ★ ☆ 等杂项符号
        (0x2700, 0x27BF),  # ✔ ✖ 等装饰符号
        (0x3000, 0x303F),  # CJK 标点 。、《》【】
        (0x3040, 0x30FF),  # 日文假名
        (0x3105, 0x312F),  # 注音
        (0xFF00, 0xFFEF),  # 全角/半角
    ]
    for lo, hi in ranges:
        points.update(range(lo, hi + 1))
    points.update(ord(c) for c in gb2312_chars())
    return sorted(points)


def main():
    if len(sys.argv) != 3:
        print(__doc__)
        return 1

    src, dst = Path(sys.argv[1]), Path(sys.argv[2])
    if not src.exists():
        print(f"源字体不存在: {src}")
        return 1

    dst.parent.mkdir(parents=True, exist_ok=True)
    unicodes = wanted()
    print(f"保留码位 {len(unicodes)} 个（含 GB2312 汉字 6763）")

    with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False, encoding="utf-8") as fh:
        fh.write("\n".join(f"U+{c:04X}" for c in unicodes))
        list_path = fh.name

    cmd = [
        sys.executable, "-m", "fontTools.subset", str(src),
        f"--unicodes-file={list_path}",
        f"--output-file={dst}",
        # 保留版权与授权记录（OFL 要求）
        "--name-IDs=0,1,2,3,4,5,6,13,14",
        # fontdue 不做 shaping，去掉布局表省体积
        "--layout-features=",
        "--drop-tables+=GSUB,GPOS,GDEF",
        "--no-hinting",
        "--desubroutinize",
    ]
    print(" ".join(cmd))
    subprocess.run(cmd, check=True)

    mb = dst.stat().st_size / 1024 / 1024
    print(f"完成: {dst} ({mb:.2f} MB, 源文件 {src.stat().st_size / 1024 / 1024:.2f} MB)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
