mod block;
mod board;
mod point;

use std::collections::BTreeSet;
use std::env;

use anyhow::{bail, Context, Result};
use getopts::Options;

use block::*;
use board::*;
use point::*;

fn try_fill_board(board: &Board, blocks: &[Block]) -> Result<(Board, Vec<(Block, Point)>)> {
    fn dfs(
        blocks: &[Block],
        available_blocks: &mut BTreeSet<usize>,
        board: &mut Board,
        cnt: &mut u32,
    ) -> Result<Vec<(Block, Point)>> {
        if available_blocks.is_empty() {
            return Ok(vec![]);
        }

        *cnt += 1;

        let p = board.first_empty_cell().unwrap();

        let block_ids = available_blocks.clone();
        for i in block_ids {
            let mut block = blocks[i].clone();
            available_blocks.remove(&i);
            for _ in 0..4 {
                block = block.rot();
                if let Err(_) = board.put_block(&p, i, &block) {
                    continue;
                }

                if let Ok(mut ans) = dfs(blocks, available_blocks, board, cnt) {
                    ans.insert(0, (block, p));
                    return Ok(ans);
                }
                board
                    .remove_block(&p, &block)
                    .expect("remove block must succeed");
            }
            available_blocks.insert(i);
        }

        bail!("solution not found");
    }

    let mut candidate_ps = vec![];
    for i in 0..board.height() {
        for j in 0..board.width() {
            if board.board[i][j] == State::Empty {
                candidate_ps.push(Point::new(i as i32, j as i32));
            }
        }
    }

    let mut mut_board = board.clone();
    let mut cnt = 0;

    let mut available_blocks = BTreeSet::new();
    for i in 0..blocks.len() {
        available_blocks.insert(i);
    }
    let r = dfs(blocks, &mut available_blocks, &mut mut_board, &mut cnt).map(|r| (mut_board, r));
    println!("Count: {}", cnt);
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill() {
        let board = Board::new(2, 2);

        let v = vec!["#.", ".#"];
        let b = Block::from_strs(&v).unwrap();
        let b1 = b.rot();
        let ans = try_fill_board(&board, &vec![b, b1]).unwrap();

        assert_eq!(ans, vec![Point::new(0, 0), Point::new(1, 0)]);
    }
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.reqopt(
        "m",
        "month",
        "month",
        "[Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec]",
    );
    opts.reqopt("d", "day", "day", "[1-31]");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", opts.short_usage(&program));
            bail!(f.to_string())
        }
    };
    if matches.opt_present("h") {
        println!("{}", opts.short_usage(&program));
        return Ok(());
    }

    const MONTH_NAMES: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let month_str: String = matches.opt_get("month").unwrap().unwrap();
    let month_pos = match MONTH_NAMES.iter().position(|m| *m == month_str) {
        None => {
            bail!("unexpected month name: {}", month_str);
        }
        Some(p) => {
            let x = if p <= 5 { 0 } else { 1 };
            let y = p - x * 5;
            Point::new(x as i32, y as i32)
        }
    };

    let day: u32 = matches
        .opt_get::<String>("day")
        .unwrap()
        .unwrap()
        .parse()
        .context("invalid number was given as day")?;
    let day_pos = {
        let x = (day - 1) / 7 + 2;
        let y = (day - 1) % 7;
        Point::new(x as i32, y as i32)
    };

    #[rustfmt::skip]
    let blocks = [
        vec![
            "###",
            "#..",
            "#..",
        ],
        vec![
            "#.#",
            "###",
        ],
        vec![
            "##.",
            ".#.",
            ".##",
        ],
        vec![
            "#.",
            "##",
            "#.",
            "#.",
        ],
        vec![
            "#.",
            "##",
            ".#",
            ".#",
        ],
        vec![
            "##",
            ".#",
            ".#",
            ".#",
        ],
        vec![
            "##.",
            "###",
        ],
        vec![
            "###",
            "###",
        ],
    ].iter().map(|b| Block::from_strs(&b)).collect::<Result<Vec<_>>>()?;

    let mut walls = [
        Point::new(0, 6),
        Point::new(1, 6),
        Point::new(6, 3),
        Point::new(6, 4),
        Point::new(6, 5),
        Point::new(6, 6),
    ]
    .iter()
    .map(|p| ('#', *p))
    .collect::<Vec<_>>();

    walls.append(&mut vec![('M', month_pos), ('D', day_pos)]);

    let board = Board::new_with_walls(7, 7, &walls);

    match try_fill_board(&board, &blocks) {
        Ok((board, _)) => {
            assert!(board.first_empty_cell().is_none());
            println!("Solution:\n{}", board);
        }
        Err(e) => {
            println!("Solution not found: {}", e);
        }
    }

    Ok(())
}