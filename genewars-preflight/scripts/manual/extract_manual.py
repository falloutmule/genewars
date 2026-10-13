from PIL import Image
from pathlib import Path
import subprocess, hashlib
root = Path(r'C:\Users\fallo\Documents\ChatGPT\Freecad\genewars-preflight')
base = root/'private'/'manual'
source = base/'Genewars_Manual_DOS_EN.pdf'
expected_sha256 = '0de3be7b3ba15d68af16744f4b742d549b840cf33162581aeb7de509bdf29309'
actual_sha256 = hashlib.sha256(source.read_bytes()).hexdigest()
if actual_sha256 != expected_sha256:
    raise SystemExit(f'Unexpected manual SHA-256: {actual_sha256}')
render_dir = base/'rendered-300'
render_dir.mkdir(parents=True, exist_ok=True)
pages = list(render_dir.glob('page-*.png'))
if len(pages) != 18:
    for path in pages:
        path.unlink()
    poppler = r'C:\Users\fallo\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\poppler\Library\bin\pdftoppm.exe'
    subprocess.run([poppler, '-f', '1', '-l', '18', '-r', '300', '-png', str(source), str(render_dir/'page')], check=True)
tess = r'C:\Program Files\Tesseract-OCR\tesseract.exe'
parts = []
for n in range(1, 19):
    path = render_dir/f'page-{n:02d}.png'
    im = Image.open(path).convert('RGB')
    w, h = im.size
    for side, box in [('left', (0,0,w//2,h)), ('right', (w//2,0,w,h))]:
        crop = im.crop(box)
        tmp = base/f'.ocr-{n:02d}-{side}.png'
        crop.save(tmp)
        run = subprocess.run([tess, str(tmp), 'stdout', '--psm', '6'], capture_output=True, text=True, encoding='utf-8', errors='replace', check=True)
        parts.append(f'\n\n===== PDF PAGE {n} / {side.upper()} HALF =====\n\n{run.stdout.strip()}')
        tmp.unlink()
text = '# GeneWars manual OCR (private working extraction)\n\n'
text += f'Source SHA-256: {actual_sha256}\n'
text += 'Rendering: Poppler 300 dpi; OCR: locally installed Tesseract, English, page segmentation mode 6. OCR is a working transcription and must be checked against page images for uncertain claims.\n'
text += ''.join(parts)
(base/'extracted-text.txt').write_text(text, encoding='utf-8')
print('pages=18, halves=36, chars=',len(text),'output=',base/'extracted-text.txt')
