"""Reconstruction checks; preserve the dated preflight rather than re-extract it."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import uuid
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = ROOT.parent
BASELINE_HASHES = {
    'game-files-manifest.json': '6139f4083031cee88bc04849f1959ae1e43aff41eef7c87cb4a62bc8b86e9e6a',
    'variant-comparison.json': 'fa63d42693a0157fa3937255237290b84d3cf6a5f4df93cd5010c66847f9588d',
}


def sha256(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    commands = []
    checks = {}

    def run(label, command, timeout=120, acceptable=(0,)):
        print('Running ' + label, flush=True)
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=timeout)
        commands.append({'label': label, 'command': command, 'exit_code': result.returncode,
                         'stdout': result.stdout, 'stderr': result.stderr})
        checks[label] = result.returncode in acceptable
        return result

    py = [sys.executable, '-B']
    run('inventory_boundary_tests', py + ['-m', 'unittest', 'discover', '-s', 'scripts', '-p', 'test_inventory.py', '-v'])
    run('rust_workspace_tests', py + ['scripts/cargo.py', 'test', '--workspace', '--offline', '--locked'], timeout=300)
    build = run('native_cli_build', py + ['scripts/cargo.py', 'build', '-p', 'genewars-tools', '--offline', '--locked'])
    if build.returncode == 0:
        run('first_campaign_resource_staging', py + ['scripts/campaign_inputs.py'])
        run('terrain_resource_staging', py + ['scripts/terrain_inputs.py'])
        run('plant_resource_staging', py + ['scripts/plant_inputs.py'])
        run('native_campaign_resource_plan', py + ['scripts/validate_campaign_plan.py'])
        run('native_terrain_cli', py + ['scripts/validate_terrain_cli.py'])
        run('native_terrain_cell_render', py + ['scripts/validate_terrain_cell_render.py'])
        run('native_terrain_view', py + ['scripts/validate_terrain_view.py'])
        run('all_rnc_records', py + ['scripts/validate_rnc_corpus.py'])
        run('real_panel_render', py + ['scripts/render_panel.py'])
        # Exact creature hashes select the independently traced packed4 path.
        # GPF alone remains blocked; BONES has its own caller-bound admission.
        run('anb_corpus_accounting', py + ['scripts/validate_sprites.py'], acceptable=(0, 1))
        sprites = json.loads((ROOT / 'research/integration/sprite-corpus.json').read_text(encoding='utf-8'))
        known = {
            'd5499fbc6989eaa87523a698bc9421af07884c3b4a6a27e70dd4d7e93fea9959': 'ANB sections are not ordered within file bounds',
        }
        actual = {row['input_sha256']: row['reason'] for row in sprites['results'] if row['status'] != 'passed'}
        checks['anb_only_documented_failures'] = actual == known and sprites['summary']['records'] == 263 and sprites['summary']['passed'] == 262
        checks['anb_explicit_codec_accounting'] = (sprites['summary']['packed4_unique_inputs'] == 26
                                                  and sprites['summary']['packed4_source_records'] == 52
                                                  and sprites['summary']['packed4_creature_unique_inputs'] == 25
                                                  and sprites['summary']['packed4_bones_source_records'] == 2
                                                  and sprites['summary']['unique_sprite_records'] == 11001)
        run('independent_creature_pixels', py + ['research/variants/creature-review.py'], timeout=180)
        run('independent_bones_pixels', py + ['research/variants/bones-review.py'])
        exe = str(ROOT / 'private/build/rust-target/debug/genewars-tools.exe')
        smoke = run('native_distance_smoke', [exe, 'distance', '0', '0', '3', '4'])
        checks['native_distance_result'] = smoke.returncode == 0 and json.loads(smoke.stdout)['squared_distance'] == 25
        bad = subprocess.run([exe, 'distance', '32768', '0', '0', '0'], capture_output=True, text=True)
        commands.append({'label': 'reject_out_of_range_input', 'exit_code': bad.returncode, 'stderr': bad.stderr})
        checks['reject_out_of_range_input'] = bad.returncode == 2 and 'signed 16-bit' in bad.stderr
    run('independent_primitive_review', py + ['research/variants/review_checks.py'])
    run('le_mapping_replay', py + ['research/variants/review_loader.py'])
    run('record_and_grid_contracts', py + ['research/variants/primitive_contract_checks.py'])
    run('renderer_bridge', py + ['scripts/ghidra/validate_renderer_bridge.py'])
    run('custom_ui_bridge', py + ['scripts/ghidra/validate_ui_bridge.py'])
    run('creature_codec_bridge', py + ['scripts/ghidra/validate_creature_bridge.py'])
    run('input_packet_bridge', py + ['scripts/ghidra/validate_input_bridge.py'])
    run('custom_list_storage', py + ['scripts/ghidra/validate_list_layout.py'])
    run('config_parser_and_writer', py + ['scripts/ghidra/validate_config_bridge.py'])
    run('english_install_copy_contract', py + ['scripts/ghidra/validate_english_install_bridge.py'])
    run('font_and_menu_contract', py + ['scripts/ghidra/validate_font_menu_bridge.py'])
    run('campaign_entry_contract', py + ['scripts/ghidra/validate_campaign_entry_bridge.py'])
    run('mission_load_contract', py + ['scripts/ghidra/validate_mission_load_bridge.py'])
    run('menu_input_contract', py + ['scripts/ghidra/validate_menu_input_bridge.py'])
    run('menu_replay_contract', py + ['scripts/ghidra/validate_menu_replay_bridge.py'])
    run('terrain_contract', py + ['scripts/ghidra/validate_terrain_bridge.py'])
    run('terrain_projection_contract', py + ['scripts/ghidra/validate_terrain_projection_bridge.py'])
    run('terrain_initial_viewport_contract', py + ['scripts/ghidra/validate_terrain_viewport_bridge.py'])
    run('entity_point_contract', py + ['scripts/ghidra/validate_entity_point_bridge.py'])
    run('plant_foreground_contract', py + ['scripts/ghidra/validate_plant_color_bridge.py'])
    run('plant_shadow_contract', py + ['scripts/ghidra/validate_plant_shadow_bridge.py'])
    run('entity_shade_consumer_contract', py + ['scripts/ghidra/validate_entity_shade_bridge.py'])
    run('entity_queue_contract', py + ['scripts/ghidra/validate_entity_queue_bridge.py'])
    run('conditional_initial_panel_contract', py + ['scripts/ghidra/validate_initial_panel_bridge.py'])
    run('mouse_motion_contract', py + ['scripts/ghidra/validate_mouse_motion_bridge.py'])
    run('plant_startup_contract', py + ['scripts/ghidra/validate_plant_startup_bridge.py'])
    run('plant_state_provider_contract', py + ['scripts/ghidra/validate_plant_state_bridge.py'])
    run('plant_subsequent_step_contract', py + ['scripts/ghidra/validate_plant_step_bridge.py'])
    run('move_order_contract', py + ['scripts/ghidra/validate_move_order_bridge.py'])
    run('player_creation_contract', py + ['scripts/ghidra/validate_player_creation_bridge.py'])
    run('actor_activation_contract', py + ['scripts/ghidra/validate_actor_activation_bridge.py'])
    run('owned_player_state', py + ['scripts/validate_player_state.py'])
    run('owned_actor_world', py + ['scripts/validate_actor_world.py'])
    run('native_creature_storage', py + ['research/variants/creature-world-review.py'])
    run('initial_creature_choice', py + ['research/variants/initial-creature-review.py'], timeout=180)
    run('owned_creature_motion', py + ['research/variants/creature-motion-review.py'], timeout=300)
    run('hungry_creature_acquisition', py + ['research/variants/creature-hungry-review.py'], timeout=180)
    run('empty_world_bookkeeping', py + ['scripts/validate_bookkeeping.py'], timeout=300)
    run('neutral_creature_scheduler', py + ['scripts/validate_family_scheduler.py'], timeout=180)
    run('neutral_plant_initialization', py + ['scripts/validate_plant_initialization.py'], timeout=180)
    run('neutral_plant_factory', py + ['scripts/validate_plant_factory.py'], timeout=180)
    run('owned_mule_water', py + ['research/variants/creature-water-review.py'], timeout=180)
    run('plant_spread_target', py + ['scripts/validate_plant_spread.py'], timeout=180)
    run('backward_target_resolver', py + ['scripts/validate_target_resolver.py'], timeout=180)
    run('independent_mule_scene', py + ['scripts/validate_mule_scene.py'], timeout=300)
    run('associated_actor_activation', py + ['scripts/validate_associated_activation.py'])
    run('actor_resource_staging', py + ['scripts/actor_inputs.py'])
    run('actor_resource_catalog', py + ['scripts/validate_actor_resources.py'])
    run('actor_environment_contract', py + ['research/variants/actor-environment-contract.py'])
    run('actor_environment_native', py + ['research/variants/actor-environment-native.py'])
    run('actor_environment_minimap', py + ['research/variants/actor-environment-minimap.py'])
    run('actor_environment_terrain_target', py + ['research/variants/actor-environment-terrain-target.py'])
    run('actor_render_independent', py + ['research/variants/actor-render-review.py'])
    run('landing_bridge', py + ['scripts/ghidra/validate_landing_bridge.py'])
    run('landing_phase1', py + ['scripts/ghidra/validate_landing_phase1.py'])
    run('landing_departure', py + ['scripts/ghidra/validate_landing_departure.py'])
    run('landing_particle', py + ['scripts/ghidra/validate_landing_particle.py'])
    run('water_effect_lifecycle', py + ['scripts/ghidra/validate_landing_water.py'])
    run('common_water_callbacks', py + ['scripts/ghidra/validate_common_water_tail.py'], timeout=180)
    run('landing_reference_cleanup', py + ['scripts/ghidra/validate_landing_lifecycle.py'])
    run('actor_pending_move', py + ['scripts/ghidra/validate_actor_motion.py'])
    run('actor_motion_animation', py + ['scripts/ghidra/validate_actor_animation.py'])
    run('actor_arrival_idle', py + ['scripts/ghidra/validate_actor_idle.py'], timeout=300)
    run('actor_callback_slices', py + ['scripts/ghidra/validate_actor_callback.py'])
    run('actor_straight_ray', py + ['scripts/ghidra/validate_actor_ray.py'])
    run('actor_player_list_order', py + ['scripts/ghidra/validate_actor_order.py'])
    run('owned_actor_motion', py + ['scripts/ghidra/validate_actor_owned.py'], timeout=300)
    run('actor_waypoint_motion', py + ['scripts/ghidra/validate_actor_waypoint.py'], timeout=300)
    run('landing_factory_world', py + ['scripts/validate_landing_world.py'])
    run('landing_steps_world', py + ['scripts/validate_landing_steps.py'])
    run('landing_effect_scheduler', py + ['scripts/validate_landing_scheduler.py'], timeout=180)
    run('blueprint_generation_and_resources', py + ['scripts/ghidra/validate_blueprint_init_bridge.py'])
    run('actor_initialization_and_creation', py + ['scripts/ghidra/validate_actor_initialization_bridge.py'])
    run('terrain_camera_target_contract', py + ['scripts/validate_camera_target.py'])
    run('terrain_factor_plan_contract', py + ['scripts/validate_terrain_factor_contract.py'])
    run('terrain_entity_shade_contract', py + ['scripts/validate_entity_shade.py'])
    run('mode5_span_contract', py + ['scripts/ghidra/validate_mode5_span_bridge.py'])
    run('mode5_triangle_contract', py + ['scripts/ghidra/validate_mode5_triangle_bridge.py'])
    run('terrain_strip_contract', py + ['scripts/ghidra/validate_terrain_strip_bridge.py'])
    run('terrain_modes_contract', py + ['scripts/ghidra/validate_terrain_modes_bridge.py'])
    terrain_build = run('native_terrain_window_build', py + ['scripts/cargo.py', 'build', '-p', 'genewars-desktop', '--bin', 'genewars-terrain', '--offline', '--locked'])
    if terrain_build.returncode == 0:
        run('native_terrain_window', py + ['scripts/validate_terrain_window.py'])
        if build.returncode == 0:
            run('native_mission_source_view', py + ['scripts/validate_mission_source_view.py'])
            run('native_mission_plant_view', py + ['scripts/validate_mission_plant_view.py'], timeout=180)
            run('native_mission_plant_scene', py + ['scripts/validate_mission_plant_scene.py'], timeout=180)
            run('native_plant_snapshot_windows', py + ['scripts/validate_plant_snapshot_window.py', '--run-window-smokes'], timeout=180)
            run('recorded_native_scene_controls', py + ['scripts/validate_scene_controls.py'])
    run('independent_terrain_cells_and_heights', py + ['research/variants/terrain-review.py'])
    run('independent_terrain_vertices', py + ['research/variants/terrain-vertex-review.py'])
    run('independent_terrain_shade_and_fade', py + ['research/variants/terrain-shade-review.py'])
    run('independent_mode5_spans', py + ['research/variants/mode5-span-review.py'])
    run('independent_mode5_triangles', py + ['research/variants/mode5-triangle-review.py'])
    run('independent_camera_targets', py + ['research/variants/camera-target-review.py'])
    run('independent_terrain_factor_plan', py + ['research/variants/terrain-factor-review.py'])
    run('independent_terrain_strip_plan', py + ['research/variants/terrain-strip-review.py'])
    run('independent_entity_shade', py + ['research/variants/entity-shade-review.py'])
    run('independent_entity_point', py + ['research/variants/entity-point-review.py'])
    run('independent_plant_passes', py + ['research/variants/plant-passes-review.py'], timeout=180)
    run('independent_entity_queue', py + ['research/variants/entity-queue-review.py'], timeout=180)
    run('independent_plant_lifecycle', py + ['research/variants/plant-lifecycle-review.py'], timeout=180)
    run('independent_player_creation', py + ['research/variants/player-creation-review.py'], timeout=180)
    run('independent_plant_scene', py + ['research/variants/plant-scene-review.py'], timeout=180)
    run('independent_conditional_initial_panel', py + ['research/variants/initial-panel-review.py'], timeout=180)
    run('independent_terrain_modes', py + ['research/variants/terrain-modes-review.py'])
    run('independent_mission_load', py + ['research/variants/mission-load-review.py'])
    run('independent_mission_source', py + ['research/variants/mission-source-review.py'])
    run('independent_viewport_context', py + ['research/variants/viewport-context-review.py'])
    run('independent_terrain_view', py + ['research/variants/terrain-view-review.py'])
    run('independent_campaign_and_menu_controls', py + ['research/variants/campaign-menu-review.py'])
    run('independent_menu_replay', py + ['research/variants/menu-replay-review.py'])
    run('bones_codec_bridge', py + ['scripts/ghidra/validate_bones_bridge.py'])
    run('construction_debit_bridge', py + ['scripts/ghidra/validate_construction_bridge.py'])
    run('independent_construction_debit', py + ['research/variants/construction-review.py'])
    run('planet_texture_bridge', py + ['scripts/ghidra/validate_planet_texture_bridge.py'])
    run('independent_planet_texture_pixels', py + ['research/variants/planet-texture-review.py'])
    run('independent_core_controls', py + ['research/variants/core-control-review.py'])
    run('windows_dependency_receipt', py + ['research/variants/windows-dependency-review.py'])
    run('sprite_exception_assessment', py + ['scripts/ghidra/assess_sprite_exceptions.py'])
    report_path = ROOT / 'private/integration' / ('native-smoke-' + uuid.uuid4().hex + '.json')
    smoke = run('native_window_smoke', py + ['scripts/desktop/smoke.py',
               str(ROOT / 'private/integration/panel/GENEPANL.RNC'),
               str(ROOT / 'private/integration/panel/MAIN.PAL'), str(report_path), '--frames', '3'])
    if smoke.returncode == 0:
        window_report = json.loads(report_path.read_text(encoding='utf-8'))
        checks['native_window_presented_three_frames'] = window_report == {'presented_frames': 3, 'exit_reason': 'frame_limit'}

    controls_build = run('native_controls_build', py + ['scripts/cargo.py', 'build', '-p', 'genewars-desktop', '--bin', 'genewars-controls', '--offline', '--locked'])
    if controls_build.returncode == 0:
        corpus = json.loads((ROOT / 'research/integration/rnc-corpus.json').read_text(encoding='utf-8'))
        control = next(row for row in corpus['results'] if row['entry_id'] == 'root/2/disc/89')
        control_path = ROOT / control['private_output']
        palette_path = ROOT / 'private/integration/panel/MAIN.PAL'
        if sha256(control_path) != 'daf41bf104dd5d6436f7af23158a5d23d624c12780e3e66177b3b5aec2fbe006':
            raise ValueError('Unexpected CD IFCUSTOM decoded bytes')
        if sha256(palette_path) != '8d9a27bca8ed203629da4ee72bb982150fae5dd876743b50a99ad90dba0f4b05':
            raise ValueError('Unexpected CD palette')
        controls_report = ROOT / 'private/integration' / ('controls-smoke-' + uuid.uuid4().hex + '.json')
        controls_smoke = run('native_controls_smoke', [str(ROOT / 'private/build/rust-target/debug/genewars-controls.exe'),
                             str(control_path), str(palette_path), '--frames', '3', '--report', str(controls_report)], timeout=30)
        if controls_smoke.returncode == 0:
            observed = json.loads(controls_report.read_text(encoding='utf-8'))
            checks['native_controls_updated_three_packets'] = (observed['packet_updates'] == 3 and observed['presented_frames'] >= 3
                                                               and observed['exit_reason'] == 'frame_limit' and 'error' not in observed)

    menu_build = run('native_menu_build', py + ['scripts/cargo.py', 'build', '-p', 'genewars-desktop', '--bin', 'genewars-menu', '--offline', '--locked'])
    if menu_build.returncode == 0:
        menu_render = run('native_menu_render_and_window', py + ['scripts/render_menu.py', '--frames', '3'], timeout=45)
        menu_review = run('independent_font_and_menu_pixels', py + ['research/variants/font-menu-review.py'])
        if menu_render.returncode == 0 and menu_review.returncode == 0:
            rendered = json.loads((ROOT / 'research/integration/menu-render.json').read_bytes())
            reviewed = json.loads((ROOT / 'research/variants/font-menu-review.json').read_bytes())
            checks['current_native_menu_pixels_match_oracle'] = (reviewed['status'] == 'passed'
                and rendered['rgb_sha256'] == reviewed['menu']['rgb_sha256'])

    runtime_build = run('native_runtime_build', py + ['scripts/cargo.py', 'build', '-p', 'genewars-desktop', '--bin', 'genewars-game', '--offline', '--locked'])
    if runtime_build.returncode == 0:
        run('runtime_resource_staging', py + ['scripts/runtime_inputs.py'])
        run('native_runtime_flow_and_failures', py + ['scripts/validate_runtime.py', '--window-smoke'], timeout=180)
        run('owned_world_reveal', py + ['scripts/validate_reveal.py'])
        run('owned_landing_runtime', py + ['scripts/validate_shared_runtime.py', '--release', '--window-smoke'], timeout=1800)
        run('independent_runtime_world', py + ['research/variants/runtime-world-review.py'], timeout=180)
        run('recorded_runtime_ui', py + ['scripts/validate_runtime_ui.py'])
    run('first_world_command_eligibility', py + ['research/variants/first-world-command-review.py'])
    run('independent_actor_command', py + ['research/variants/actor-command-review.py'], timeout=180)
    run('independent_actor_activation', py + ['research/variants/actor-activation-review.py'], timeout=180)

    baseline = json.loads((ROOT / 'research/verification.json').read_text(encoding='utf-8'))
    checks['dated_preflight_twelve_checks_passed'] = baseline['all_passed'] and len(baseline['checks']) == 12 and all(baseline['checks'].values())
    checks['dated_manifest_and_comparison_unchanged'] = all(sha256(ROOT / 'research' / name) == value for name, value in BASELINE_HASHES.items())
    manifest = json.loads((ROOT / 'research/game-files-manifest.json').read_text(encoding='utf-8'))
    checks['original_archive_unchanged'] = sha256(Path(manifest['source']['path'])) == manifest['source']['planning_sha256']
    checks['manifest_assertions_hold'] = all(manifest['checks'].values())

    git = ['git', '-C', str(WORKSPACE), '-c', 'safe.directory=' + WORKSPACE.as_posix()]
    tracked = subprocess.run(git + ['ls-files', '--', 'genewars-preflight/private'], capture_output=True, text=True)
    checks['no_private_files_tracked'] = tracked.returncode == 0 and not tracked.stdout.strip()
    private_files = sorted(p.relative_to(WORKSPACE).as_posix() for p in (ROOT / 'private').rglob('*') if p.is_file())
    ignored = subprocess.run(git + ['check-ignore', '--no-index', '-z', '--stdin'],
                             input='\0'.join(private_files) + '\0', capture_output=True, text=True)
    checks['all_private_files_ignored'] = ignored.returncode == 0 and set(ignored.stdout.rstrip('\0').split('\0')) == set(private_files)
    record = {
        'schema_version': 1, 'completed_utc': datetime.now(timezone.utc).isoformat(),
        'all_passed': all(checks.values()), 'checks': checks, 'commands': commands,
        'private_files_checked': len(private_files), 'baseline_sha256': BASELINE_HASHES,
        'source_archive_sha256': manifest['source']['planning_sha256'],
        'scope': 'Rust/parser/static checks, real panel rendering, native asset/control-window presentation and dated-preflight preservation. Strict metadata bounds and separate hash-bound packed4 creature/BONES admissions supersede the unsafe full-file sprite result. One GPF ANB record remains unresolved. The earlier 12 checks are historical evidence; no broad re-extraction was performed. This verifier does not execute original programs; separately approved reference observations and supervised native-input tests have their own receipts. No gameplay parity or playable game is claimed here.',
    }
    destination = ROOT / 'research/integration/verification.json'
    destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'all_passed': record['all_passed'], 'checks': checks, 'private_files_checked': len(private_files)}))
    return 0 if record['all_passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
