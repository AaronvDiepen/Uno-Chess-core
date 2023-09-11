use crate::bitboard::{BitBoard, EMPTY};
use crate::board::Board;
use crate::color::Color;
use crate::movegen::{MoveList, SquareAndBitBoard};
use crate::piece::Piece;
use crate::square::Square;

use crate::magic::{
    get_bishop_moves, get_king_moves, get_knight_moves, get_pawn_attacks, get_pawn_moves, get_rook_moves,
};

pub trait PieceType {
    fn is(piece: Piece) -> bool;
    fn into_piece() -> Piece;
    #[inline(always)]
    fn captures(src: Square, color: Color, combined: BitBoard, board: &Board) -> BitBoard
    {
        (
            (get_bishop_moves(src, combined) & (board.pieces(Piece::Bishop) | board.pieces(Piece::Queen)))
            | (get_rook_moves(src, combined) & (board.pieces(Piece::Rook) | board.pieces(Piece::Queen)))
            | (get_pawn_attacks(src, color, *board.pieces(Piece::Pawn)))
            | (get_knight_moves(src) & board.pieces(Piece::Knight))
        ) & board.color_combined(!color)
    }
    fn pseudo_legals(src: Square, color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard;
    #[inline(always)]
    fn legals<T>(movelist: &mut MoveList, board: &Board, mask: BitBoard)
    where
        T: CheckType,
    {
        let combined = board.combined();
        let color = board.side_to_move();
        let pieces = board.pieces(Self::into_piece()) & board.color_combined(color);
        let checkers = board.checkers();

        if T::IN_CHECK {
            for src in pieces {
                let moves = (Self::pseudo_legals(src, color, *combined, mask) | Self::captures(src, color, *combined, &board)) & checkers;
                if moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                    }
                }
            }
        } else {
            for src in pieces {
                let moves = Self::pseudo_legals(src, color, *combined, mask) | Self::captures(src, color, *combined, &board);
                if moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                    }
                }
            }
        }
    }
}

pub struct PawnType;
pub struct BishopType;
pub struct KnightType;
pub struct RookType;
pub struct QueenType;
pub struct KingType;

pub trait CheckType {
    const IN_CHECK: bool;
}

pub struct InCheckType;
pub struct NotInCheckType;

impl CheckType for InCheckType {
    const IN_CHECK: bool = true;
}

impl CheckType for NotInCheckType {
    const IN_CHECK: bool = false;
}

impl PieceType for PawnType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Pawn
    }

    fn into_piece() -> Piece {
        Piece::Pawn
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_pawn_moves(src, color, combined) & mask
    }

    #[inline(always)]
    fn legals<T>(movelist: &mut MoveList, board: &Board, mask: BitBoard)
    where
        T: CheckType,
    {
        let combined = board.combined();
        let color = board.side_to_move();
        let pieces = board.pieces(Self::into_piece()) & board.color_combined(color);
        let checkers = board.checkers();

        if T::IN_CHECK {
            for src in pieces {
                let moves = (Self::pseudo_legals(src, color, *combined, mask) | Self::captures(src, color, *combined, &board)) & checkers;
                let promotions = color.to_promotion_board();
                let normal_moves = moves & !promotions;
                let promotion_moves = moves & promotions;

                if normal_moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(
                            src,
                            normal_moves,
                            false,
                        ));
                    }
                }
                if promotion_moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(
                            src,
                            promotion_moves,
                            true,
                        ));
                    }
                }
            }
        } else {
            for src in pieces {
                let moves = Self::pseudo_legals(src, color, *combined, mask) | Self::captures(src, color, *combined, &board);
                let promotions = color.to_promotion_board();
                let normal_moves = moves & !promotions;
                let promotion_moves = moves & promotions;

                if normal_moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(
                            src,
                            normal_moves,
                            false,
                        ));
                    }
                }
                if promotion_moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(
                            src,
                            promotion_moves,
                            true,
                        ));
                    }
                }
            }
        }
    }
}

impl PieceType for BishopType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Bishop
    }

    fn into_piece() -> Piece {
        Piece::Bishop
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_bishop_moves(src, combined) & mask
    }
}

impl PieceType for KnightType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Knight
    }

    fn into_piece() -> Piece {
        Piece::Knight
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, _combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_knight_moves(src) & mask
    }
}

impl PieceType for RookType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Rook
    }

    fn into_piece() -> Piece {
        Piece::Rook
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_rook_moves(src, combined) & mask
    }
}

impl PieceType for QueenType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Queen
    }

    fn into_piece() -> Piece {
        Piece::Queen
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        (get_rook_moves(src, combined) ^ get_bishop_moves(src, combined)) & mask
    }
}

impl KingType {
    /// Is a particular king move legal?
    #[inline(always)]
    pub fn legal_king_move(board: &Board, dest: Square) -> bool {
        get_king_moves(dest) & board.color_combined(!board.side_to_move()) == EMPTY
    }
}

impl PieceType for KingType {
    fn is(piece: Piece) -> bool {
        piece == Piece::King
    }

    fn into_piece() -> Piece {
        Piece::King
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, _combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_king_moves(src) & mask
    }

    #[inline(always)]
    fn legals<T>(movelist: &mut MoveList, board: &Board, mask: BitBoard)
    where
        T: CheckType,
    {
        let combined = board.combined();
        let color = board.side_to_move();
        let ksq = board.king_square(color);

        let mut moves = Self::pseudo_legals(ksq, color, *combined, mask) | Self::captures(ksq, color, *combined, &board);

        let copy = moves;
        for dest in copy {
            if !KingType::legal_king_move(board, dest) {
                moves ^= BitBoard::from_square(dest);
            }
        }

        // If we are not in check, we may be able to castle.
        // We can do so iff:
        //  * the `Board` structure says we can.
        //  * the squares between my king and my rook are empty.
        //  * no enemy pieces are attacking the squares between the king, and the kings
        //    destination square.
        //  ** This is determined by going to the left or right, and calling
        //     'legal_king_move' for that square.
        if !T::IN_CHECK {
            if board.my_castle_rights().has_kingside()
                && (combined & board.my_castle_rights().kingside_squares(color)) == EMPTY
            {
                let middle = ksq.uright();
                let right = middle.uright();
                if KingType::legal_king_move(board, middle)
                    && KingType::legal_king_move(board, right)
                {
                    moves ^= BitBoard::from_square(right);
                }
            }

            if board.my_castle_rights().has_queenside()
                && (combined & board.my_castle_rights().queenside_squares(color)) == EMPTY
            {
                let middle = ksq.uleft();
                let left = middle.uleft();
                if KingType::legal_king_move(board, middle)
                    && KingType::legal_king_move(board, left)
                {
                    moves ^= BitBoard::from_square(left);
                }
            }
        }

        if moves != EMPTY {
            unsafe {
                movelist.push_unchecked(SquareAndBitBoard::new(ksq, moves, false));
            }
        }
    }
}
