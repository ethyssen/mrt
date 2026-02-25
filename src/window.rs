use std::process::Command;

use anyhow::{Context, Result};

struct WorkArea {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

struct FrameExtents {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

/// Parse the work area from `wmctrl -d` (accounts for panels/docks).
/// Format: `0  * DG: 3200x1800  VP: 0,0  WA: 70,27 3130x1773  Workspace 1`
fn work_area() -> Result<WorkArea> {
    let output = Command::new("wmctrl")
        .args(["-d"])
        .output()
        .context("failed to run wmctrl -d")?;

    let text = String::from_utf8_lossy(&output.stdout);
    // Find the current desktop (marked with *)
    let line = text
        .lines()
        .find(|l| l.contains(" * "))
        .context("no active desktop found")?;

    let wa_pos = line.find("WA: ").context("no WA field")? + 4;
    let wa_str = &line[wa_pos..];
    // "70,27 3130x1773  Workspace 1"
    let parts: Vec<&str> = wa_str.split_whitespace().collect();
    let origin: Vec<i32> = parts[0]
        .split(',')
        .map(|s| s.parse())
        .collect::<std::result::Result<_, _>>()?;
    let size: Vec<i32> = parts[1]
        .split('x')
        .map(|s| s.parse())
        .collect::<std::result::Result<_, _>>()?;

    Ok(WorkArea {
        x: origin[0],
        y: origin[1],
        w: size[0],
        h: size[1],
    })
}

/// Get GTK client-side decoration frame extents for a window.
/// These are invisible shadow borders that wmctrl counts as part of the window.
fn frame_extents(window_id: &str) -> FrameExtents {
    let output = Command::new("xprop")
        .args(["-id", window_id, "_GTK_FRAME_EXTENTS"])
        .output();

    if let Ok(output) = output {
        let text = String::from_utf8_lossy(&output.stdout);
        // "_GTK_FRAME_EXTENTS(CARDINAL) = 26, 26, 23, 29"
        if let Some(pos) = text.find("= ") {
            let vals: Vec<i32> = text[pos + 2..]
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            if vals.len() == 4 {
                return FrameExtents {
                    left: vals[0],
                    right: vals[1],
                    top: vals[2],
                    bottom: vals[3],
                };
            }
        }
    }

    FrameExtents { left: 0, right: 0, top: 0, bottom: 0 }
}

fn active_window_id() -> Result<String> {
    let output = Command::new("xdotool")
        .args(["getactivewindow"])
        .output()
        .context("failed to run xdotool getactivewindow")?;
    let dec_id: u64 = String::from_utf8_lossy(&output.stdout).trim().parse()?;
    Ok(format!("0x{dec_id:08x}"))
}

/// Wait for a window with the given title to appear, then return its ID.
/// Polls every 250ms for up to 10 seconds.
fn window_id_by_title(title: &str) -> Result<String> {
    use std::thread;
    use std::time::{Duration, Instant};

    let deadline = Instant::now() + Duration::from_secs(10);

    loop {
        let output = Command::new("xdotool")
            .args(["search", "--name", title])
            .output()
            .context("failed to run xdotool search")?;

        let text = String::from_utf8_lossy(&output.stdout);
        if let Some(first_line) = text.lines().next() {
            if let Ok(dec_id) = first_line.trim().parse::<u64>() {
                return Ok(format!("0x{dec_id:08x}"));
            }
        }

        if Instant::now() >= deadline {
            anyhow::bail!("timed out waiting for window with title: {title}");
        }

        thread::sleep(Duration::from_millis(250));
    }
}

/// Place a window at the given work-area-relative rect, compensating for CSD frame extents.
fn place_window(selector: &str, wid: &str, wa: &WorkArea, wa_x: i32, wa_y: i32, wa_w: i32, wa_h: i32) -> Result<()> {
    let f = frame_extents(wid);

    // wmctrl -e coordinates include the invisible CSD border.
    // To make the *visible* part fill the target rect, we expand outward by the frame extents.
    let x = wa.x + wa_x - f.left;
    let y = wa.y + wa_y - f.top;
    let w = wa_w + f.left + f.right;
    let h = wa_h + f.top + f.bottom;

    // Remove any maximized state first so wmctrl -e works
    let _ = Command::new("wmctrl")
        .args(["-r", selector, "-b", "remove,maximized_vert,maximized_horz"])
        .status();

    Command::new("wmctrl")
        .args(["-r", selector, "-e", &format!("0,{x},{y},{w},{h}")])
        .status()
        .context("failed to run wmctrl -e")?;

    Ok(())
}

/// Snap the currently active window to the right half of the screen.
pub fn snap_active_right() -> Result<()> {
    let wa = work_area()?;
    let wid = active_window_id()?;
    let half = wa.w / 2;
    place_window(":ACTIVE:", &wid, &wa, half, 0, wa.w - half, wa.h)
}

/// Snap a window (found by title substring) to the left half of the screen.
pub fn snap_window_left(title: &str) -> Result<()> {
    let wa = work_area()?;
    let wid = window_id_by_title(title)?;
    let half = wa.w / 2;
    place_window(title, &wid, &wa, 0, 0, half, wa.h)
}
