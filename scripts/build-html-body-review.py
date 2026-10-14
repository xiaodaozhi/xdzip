#!/usr/bin/env python3
"""Bundle current generated review evidence without retaining it in Git."""
import argparse
from pathlib import Path
from visual_review import build_review
ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--source', type=Path, default=ROOT/'build/review/html-body')
parser.add_argument('--output', type=Path, default=ROOT/'dist/xdzip-body-review.html')
args = parser.parse_args()
build_review(args.source, args.output, '内容区域原稿与实际实现')
