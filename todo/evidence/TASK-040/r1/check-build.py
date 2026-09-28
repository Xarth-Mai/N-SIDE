"""Inspect generated guide markup and search records without claiming browser rendering."""
import hashlib
import json
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urljoin


class MainText(HTMLParser):
    def __init__(self):
        super().__init__()
        self.in_main = False
        self.text = []
        self.links = []

    def handle_starttag(self, tag, attrs):
        if tag == "main":
            self.in_main = True
        if self.in_main and tag == "a":
            self.links.extend(value for key, value in attrs if key == "href")

    def handle_endtag(self, tag):
        if tag == "main":
            self.in_main = False

    def handle_data(self, data):
        if self.in_main:
            self.text.append(data)


root = Path(__file__).resolve().parents[4]
site = root / "docs/.vitepress/dist-player"
page = site / "player/guide/index.html"
parser = MainText()
parser.feed(page.read_text())
text = "".join(parser.text)
for expected in ["游玩指南", "--walk-preview", "./run-walk-preview.sh", "125%", "65%", "三处回望台", "并不是一份从日常到委托结尾的完整 Demo"]:
    assert expected in text, f"Missing guide content: {expected}"
links = [urljoin("/player/guide/", href) for href in parser.links]
assert "/player/locations/null-site" in links
assert not any(href.startswith("/dev/") for href in links)
search, = (site / "assets").glob("search-index.*.json")
records = json.loads(search.read_text())["documentIds"].values()
guide_records = [record for record in records if record.startswith("/player/guide/#")]
assert len(guide_records) == 10, guide_records
for heading in ["游玩指南", "基本操作", "文字大小与镜头灵敏度", "第一次可以这样走"]:
    assert f"/player/guide/#{heading}" in guide_records
print(json.dumps({
    "result": "PASS",
    "scope": "generated HTML main content, player links, and search records; no browser rendering",
    "guide_search_records": guide_records,
    "outputs": [{"path": str(path.relative_to(root)), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in [page, search]],
}, ensure_ascii=False, indent=2))
