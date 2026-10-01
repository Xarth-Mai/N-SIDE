"""Compatibility entry point for the shared building-normal baker."""
from pathlib import Path
import runpy

if __name__ == "__main__":
    root = Path(__file__).resolve().parents[3]
    runpy.run_path(str(root / "source-assets/environment-kit/materials/bake_normals.py"), run_name="__main__")
