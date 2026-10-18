"""Run004 staged/hash/default-no-process validation; no GUI or emulator launch."""
from pathlib import Path
import hashlib
import validate_run003

if __name__=='__main__':
    root=Path(__file__).resolve().parents[2]
    source=root/'private/review/input-free-yield/src-gui-sdlmain.cpp'
    assert hashlib.sha256(source.read_bytes()).hexdigest()=='0e2978912e692f9e016c67f15fc4e8e54de69ab481ba5c73a4c8edc086a8bf6c'
    text=source.read_text()
    assert 'Add_bool("raw_mouse_input", Property::Changeable::OnlyAtStart, false)' in text
    assert 'SDL_HINT_MOUSE_RELATIVE_MODE_WARP, section->Get_bool("raw_mouse_input") ? "0" : "1"' in text
    validate_run003.main('run-004')
