use std::rc::{Rc, Weak};

struct Node {
    children: Vec<(ChessMove, Rc<Node>)>,
    score: u16,
    depth: u16,
}

struct Tree {
    root: Rc<Node>,
    map: HashMap<Game, Weak<Node>>
}

impl Tree {
    pub fn new(game: Game) -> Tree {
        let root = Rc::new(Node{children: vec!(), game: game.clone(), score: Engine.evaluate_state(&game), depth: 0});
        let tree = Tree{root:Rc::clone(root), map: HashMap::new()};
        tree.map.insert(game, root.downgrade());

        tree
    }

    pub fn advance_move(&mut self, chess_move: ChessMove) -> bool {
        let _ = self.map.remove(&self.tree.game);

        match self.root.children.iter().find(|(cm, _)| cm == chess_move) {
            Some(next_node) => {
                self.root = next_node;
                true
            }
            None => {
                false
            }
        }
    }
}