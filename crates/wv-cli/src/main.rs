use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::time::{Duration, Instant};
use wv_core::fighter::{FighterData, FighterId};
use wv_core::game::{GamePhase, GameState};
use wv_core::input::InputState;
use wv_core::physics::{ARENA_MIN_X, ARENA_MAX_X};
use wv_core::state_machine::FighterState;

const TARGET_FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS);

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let result = run_game(&mut stdout);

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    result
}

fn run_game(stdout: &mut io::Stdout) -> io::Result<()> {
    let mut game = GameState::new_in_select();
    let mut p1_selection: usize = 0;
    let mut p2_selection: usize = 1;
    let mut select_phase: u8 = 0; // 0 = P1 selecting, 1 = P2 selecting

    loop {
        let frame_start = Instant::now();

        // Collect input
        let mut p1_input = InputState::default();
        let mut p2_input = InputState::default();
        let mut quit = false;

        // Poll all available events
        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL))
                {
                    quit = true;
                    break;
                }

                match game.phase {
                    GamePhase::FighterSelect => {
                        handle_select_input(
                            key,
                            &mut p1_selection,
                            &mut p2_selection,
                            &mut select_phase,
                            &mut game,
                        );
                    }
                    GamePhase::MatchOver => {
                        if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                            game = GameState::new_in_select();
                            p1_selection = 0;
                            p2_selection = 1;
                            select_phase = 0;
                        }
                    }
                    _ => {
                        apply_key_to_input(key, &mut p1_input, &mut p2_input);
                    }
                }
            }
        }

        if quit {
            break;
        }

        game.tick(&p1_input, &p2_input);
        render(stdout, &game, p1_selection, p2_selection, select_phase)?;

        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - elapsed);
        }
    }

    Ok(())
}

fn handle_select_input(
    key: KeyEvent,
    p1_sel: &mut usize,
    p2_sel: &mut usize,
    phase: &mut u8,
    game: &mut GameState,
) {
    let fighters = FighterId::ALL;

    if *phase == 0 {
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                *p1_sel = p1_sel.checked_sub(1).unwrap_or(fighters.len() - 1);
            }
            KeyCode::Char('s') | KeyCode::Down => {
                *p1_sel = (*p1_sel + 1) % fighters.len();
            }
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('j') => {
                *phase = 1;
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Up => {
                *p2_sel = p2_sel.checked_sub(1).unwrap_or(fighters.len() - 1);
            }
            KeyCode::Down => {
                *p2_sel = (*p2_sel + 1) % fighters.len();
            }
            KeyCode::Enter | KeyCode::Char('0') => {
                game.select_fighters(fighters[*p1_sel], fighters[*p2_sel]);
            }
            _ => {}
        }
    }
}

fn apply_key_to_input(key: KeyEvent, p1: &mut InputState, p2: &mut InputState) {
    match key.code {
        // Player 1: WASD + JKL + Space + Tab
        KeyCode::Char('w') => p1.move_forward = true,
        KeyCode::Char('s') => p1.move_back = true,
        KeyCode::Char('a') => p1.move_left = true,
        KeyCode::Char('d') => p1.move_right = true,
        KeyCode::Char('j') => p1.light_attack = true,
        KeyCode::Char('k') => p1.heavy_attack = true,
        KeyCode::Char('l') => p1.special = true,
        KeyCode::Char(' ') => p1.block = true,
        KeyCode::Tab => p1.dash = true,

        // Player 2: Arrows + ,./ + 0 + backslash
        KeyCode::Up => p2.move_forward = true,
        KeyCode::Down => p2.move_back = true,
        KeyCode::Left => p2.move_left = true,
        KeyCode::Right => p2.move_right = true,
        KeyCode::Char(',') => p2.light_attack = true,
        KeyCode::Char('.') => p2.heavy_attack = true,
        KeyCode::Char('/') => p2.special = true,
        KeyCode::Char('0') => p2.block = true,
        KeyCode::Char('\\') => p2.dash = true,

        _ => {}
    }
}

fn render(
    stdout: &mut io::Stdout,
    game: &GameState,
    p1_sel: usize,
    p2_sel: usize,
    sel_phase: u8,
) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

    match game.phase {
        GamePhase::FighterSelect => render_select(stdout, p1_sel, p2_sel, sel_phase),
        GamePhase::Countdown => render_countdown(stdout, game),
        GamePhase::Fighting => render_fight(stdout, game),
        GamePhase::RoundOver => render_round_over(stdout, game),
        GamePhase::MatchOver => render_match_over(stdout, game),
    }
}

fn render_select(
    stdout: &mut io::Stdout,
    p1_sel: usize,
    p2_sel: usize,
    phase: u8,
) -> io::Result<()> {
    let fighters = FighterId::ALL;

    write!(stdout, "\r\n")?;
    write!(stdout, "  ╔══════════════════════════════════════════════════════════╗\r\n")?;
    write!(stdout, "  ║              WAR VILLAGE - SELECT FIGHTER               ║\r\n")?;
    write!(stdout, "  ╚══════════════════════════════════════════════════════════╝\r\n")?;
    write!(stdout, "\r\n")?;

    if phase == 0 {
        write!(stdout, "  >> PLAYER 1: Choose your fighter (W/S + Enter)\r\n")?;
    } else {
        write!(stdout, "  Player 1: {}\r\n", fighters[p1_sel])?;
        write!(stdout, "  >> PLAYER 2: Choose your fighter (Up/Down + Enter)\r\n")?;
    }
    write!(stdout, "\r\n")?;

    let active_sel = if phase == 0 { p1_sel } else { p2_sel };

    for (i, fighter_id) in fighters.iter().enumerate() {
        let data = FighterData::get(*fighter_id);
        let marker = if i == active_sel { ">>" } else { "  " };
        write!(
            stdout,
            "  {} {:<8} | {:?} | HP:{:.0} SPD:{:.1} DEF:{:.2} | {}\r\n",
            marker, fighter_id, data.style, data.max_health, data.move_speed, data.defense,
            data.default_weapon,
        )?;
    }

    write!(stdout, "\r\n  [ESC] Quit\r\n")?;
    stdout.flush()
}

fn render_countdown(stdout: &mut io::Stdout, game: &GameState) -> io::Result<()> {
    render_hud(stdout, game)?;
    write!(stdout, "\r\n")?;
    write!(stdout, "                    === {} ===\r\n", game.countdown_display())?;
    write!(stdout, "\r\n")?;
    write!(stdout, "         Round {} of Best-of-3\r\n", game.current_round)?;
    stdout.flush()
}

fn render_fight(stdout: &mut io::Stdout, game: &GameState) -> io::Result<()> {
    render_hud(stdout, game)?;
    write!(stdout, "\r\n")?;
    render_arena(stdout, game)?;
    write!(stdout, "\r\n")?;
    render_state_info(stdout, game)?;
    write!(stdout, "\r\n")?;

    if let Some(ref info) = game.last_hit_info {
        write!(stdout, "  HIT: {}\r\n", info)?;
    }

    write!(stdout, "\r\n")?;
    render_controls(stdout)?;
    stdout.flush()
}

fn render_hud(stdout: &mut io::Stdout, game: &GameState) -> io::Result<()> {
    let p1 = &game.fighters[0];
    let p2 = &game.fighters[1];

    write!(stdout, "\r\n")?;
    write!(
        stdout,
        "  Round {}  |  Time: {:.0}s  |  Wins: P1[{}] - P2[{}]\r\n",
        game.current_round,
        game.round_time_remaining(),
        p1.round_wins,
        p2.round_wins,
    )?;
    write!(stdout, "\r\n")?;
    write!(
        stdout,
        "  P1: {:<12}                     P2: {}\r\n",
        p1.data.id, p2.data.id,
    )?;

    let p1_bar = health_bar(p1.health_pct(), 20);
    let p2_bar = health_bar(p2.health_pct(), 20);
    write!(
        stdout,
        "  HP [{}] {:<6.1}    HP [{}] {:.1}\r\n",
        p1_bar, p1.health, p2_bar, p2.health,
    )?;

    let p1_stam = stamina_bar(p1.stamina_pct(), 20);
    let p2_stam = stamina_bar(p2.stamina_pct(), 20);
    write!(
        stdout,
        "  SP [{}] {:<6.1}    SP [{}] {:.1}\r\n",
        p1_stam, p1.stamina, p2_stam, p2.stamina,
    )?;

    Ok(())
}

fn health_bar(pct: f32, width: usize) -> String {
    let filled = (pct * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "#".repeat(filled), "-".repeat(empty))
}

fn stamina_bar(pct: f32, width: usize) -> String {
    let filled = (pct * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "=".repeat(filled), "-".repeat(empty))
}

fn render_arena(stdout: &mut io::Stdout, game: &GameState) -> io::Result<()> {
    let arena_width: usize = 60;
    let p1_pos = &game.fighters[0].physics.position;
    let p2_pos = &game.fighters[1].physics.position;

    let range = ARENA_MAX_X - ARENA_MIN_X;
    let p1_col = ((p1_pos.x - ARENA_MIN_X) / range * (arena_width - 1) as f32)
        .round()
        .clamp(0.0, (arena_width - 1) as f32) as usize;
    let p2_col = ((p2_pos.x - ARENA_MIN_X) / range * (arena_width - 1) as f32)
        .round()
        .clamp(0.0, (arena_width - 1) as f32) as usize;

    write!(stdout, "  +{}+\r\n", "-".repeat(arena_width))?;

    let p1_air = !game.fighters[0].physics.grounded;
    let p2_air = !game.fighters[1].physics.grounded;

    if p1_air || p2_air {
        let mut air_row = vec![' '; arena_width];
        if p1_air {
            air_row[p1_col] = '1';
        }
        if p2_air {
            air_row[p2_col] = '2';
        }
        let air_str: String = air_row.into_iter().collect();
        write!(stdout, "  |{}|\r\n", air_str)?;
    }

    let mut ground = vec![' '; arena_width];
    let p1_char = fighter_char(&game.fighters[0], '1');
    let p2_char = fighter_char(&game.fighters[1], '2');

    if !p1_air {
        ground[p1_col] = p1_char;
    }
    if !p2_air {
        if p1_col == p2_col && !p1_air {
            ground[p2_col] = 'X';
        } else {
            ground[p2_col] = p2_char;
        }
    }

    let ground_str: String = ground.into_iter().collect();
    write!(stdout, "  |{}|\r\n", ground_str)?;
    write!(stdout, "  +{}+\r\n", "=".repeat(arena_width))?;

    Ok(())
}

fn fighter_char(fighter: &wv_core::game::Fighter, default: char) -> char {
    match fighter.state_machine.state {
        FighterState::Attacking => 'A',
        FighterState::Blocking => 'B',
        FighterState::Dashing => 'D',
        FighterState::HitStun => 'H',
        FighterState::Knockdown => '_',
        FighterState::GettingUp => '^',
        _ => default,
    }
}

fn render_state_info(stdout: &mut io::Stdout, game: &GameState) -> io::Result<()> {
    let p1 = &game.fighters[0];
    let p2 = &game.fighters[1];

    write!(
        stdout,
        "  P1: {:<12} Facing: {:<6}       P2: {:<12} Facing: {}\r\n",
        p1.state_machine.state,
        if p1.facing == wv_core::types::Facing::Right { "Right" } else { "Left" },
        p2.state_machine.state,
        if p2.facing == wv_core::types::Facing::Right { "Right" } else { "Left" },
    )?;
    write!(
        stdout,
        "  Pos: ({:.1}, {:.1})                      Pos: ({:.1}, {:.1})\r\n",
        p1.physics.position.x, p1.physics.position.y,
        p2.physics.position.x, p2.physics.position.y,
    )?;

    Ok(())
}

fn render_round_over(stdout: &mut io::Stdout, game: &GameState) -> io::Result<()> {
    render_hud(stdout, game)?;
    write!(stdout, "\r\n")?;

    let p1_dead = !game.fighters[0].is_alive();
    let p2_dead = !game.fighters[1].is_alive();

    if p1_dead && p2_dead {
        write!(stdout, "          === DOUBLE KO! ===\r\n")?;
    } else if p1_dead {
        write!(stdout, "          === PLAYER 2 WINS THE ROUND! ===\r\n")?;
    } else if p2_dead {
        write!(stdout, "          === PLAYER 1 WINS THE ROUND! ===\r\n")?;
    } else if game.fighters[0].health > game.fighters[1].health {
        write!(stdout, "          === TIME! PLAYER 1 WINS! ===\r\n")?;
    } else if game.fighters[1].health > game.fighters[0].health {
        write!(stdout, "          === TIME! PLAYER 2 WINS! ===\r\n")?;
    } else {
        write!(stdout, "          === TIME! DRAW! ===\r\n")?;
    }

    stdout.flush()
}

fn render_match_over(stdout: &mut io::Stdout, game: &GameState) -> io::Result<()> {
    write!(stdout, "\r\n")?;
    write!(stdout, "  ╔══════════════════════════════════════════════════════════╗\r\n")?;

    if let Some(winner) = game.winner() {
        let name = &game.fighters[winner].data.id;
        write!(
            stdout,
            "  ║    PLAYER {} ({}) WINS THE MATCH!                        ║\r\n",
            winner + 1, name,
        )?;
    }

    write!(stdout, "  ╚══════════════════════════════════════════════════════════╝\r\n")?;
    write!(stdout, "\r\n")?;
    write!(
        stdout,
        "  Final Score: P1 [{}] - P2 [{}]\r\n",
        game.fighters[0].round_wins, game.fighters[1].round_wins,
    )?;
    write!(stdout, "\r\n")?;
    write!(stdout, "  [Enter/Space] Play Again   [ESC] Quit\r\n")?;
    stdout.flush()
}

fn render_controls(stdout: &mut io::Stdout) -> io::Result<()> {
    write!(stdout, "  ─── Controls ────────────────────────────────────\r\n")?;
    write!(stdout, "  P1: WASD=Move  J=Light K=Heavy L=Special\r\n")?;
    write!(stdout, "      Space=Block  Tab=Dash\r\n")?;
    write!(stdout, "  P2: Arrows=Move  ,=Light .=Heavy /=Special\r\n")?;
    write!(stdout, "      0=Block  \\=Dash\r\n")?;
    write!(stdout, "  [ESC] Quit\r\n")?;
    Ok(())
}
