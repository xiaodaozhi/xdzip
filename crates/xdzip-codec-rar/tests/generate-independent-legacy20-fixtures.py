"""Generate owned content with an explicitly supplied official RAR2.90 writer.

No author executable is downloaded, installed or vendored by this recipe. The
small checked-in archives remain usable without running this optional script.
Archive timestamps can differ; their full original contents are deterministic.
"""

import argparse
import hashlib
import json
import pathlib
import subprocess

AUTHOR_SHA256 = "3f938b2f4cc13b9b85d61417b36af45a970dbee0e09e9d9e13f4e00350485112"


def inputs():
    first = bytes((31 * i + 7 * (i >> 5) + 13) & 255 for i in range(65537))
    second = first[173:] + first[:173] + first[:32780]
    third = bytes((i * i + 5 * i + (i >> 3)) & 255 for i in range(33333))
    return {"first.bin": first, "second.bin": second, "third.bin": third}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--author", type=pathlib.Path, required=True)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    args = parser.parse_args()
    author = args.author.resolve(strict=True)
    assert hashlib.sha256(author.read_bytes()).hexdigest() == AUTHOR_SHA256
    output = args.output.resolve()
    repository = pathlib.Path(__file__).resolve().parents[3]
    assert output.is_relative_to(repository / "build"), "output must stay below repository build/"
    output.mkdir(parents=True, exist_ok=True)
    known = inputs()
    for name, data in known.items():
        (output / name).write_bytes(data)
    records = []
    for label, switches in [
        ("plain", ["-s-"]),
        ("solid", ["-s"]),
        ("audio-solid", ["-s", "-mmf"]),
    ]:
        archive = output / (label + ".rar")
        if archive.exists():
            raise FileExistsError(archive)
        command = ["a", "-cfg-", "-inul", "-y", "-m5", "-md1024", *switches, archive.name, *known]
        made = subprocess.run([str(author), *command], cwd=output, capture_output=True, timeout=60)
        assert made.returncode == 0
        checked = subprocess.run(
            [str(author), "t", "-cfg-", "-inul", archive.name],
            cwd=output, capture_output=True, timeout=60,
        )
        assert checked.returncode == 0
        for name, expected in known.items():
            actual = subprocess.run(
                [str(author), "p", "-cfg-", "-inul", archive.name, name],
                cwd=output, capture_output=True, timeout=60,
            )
            assert actual.returncode == 0 and actual.stdout == expected
        packed = archive.read_bytes()
        assert len(packed) <= 32768
        records.append({"file": archive.name, "command": command, "bytes": len(packed), "sha256": hashlib.sha256(packed).hexdigest()})
    (output / "author-results.json").write_text(
        json.dumps({"author_sha256": AUTHOR_SHA256, "fixtures": records}, indent=2) + "\n",
        encoding="utf-8",
    )
    print("Three bounded archives generated and all original content verified.")


if __name__ == "__main__":
    main()
