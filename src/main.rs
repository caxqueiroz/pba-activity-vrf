use rand::{rngs::OsRng, Rng};
use schnorrkel::{Keypair};
use schnorrkel::vrf::{VRFInOut, VRFProof};
use schnorrkel::signing_context;

#[derive(Clone, PartialEq)]
pub struct Card {
    rank: String,
    suit: String,
}

impl Card {
    fn new(rank: String, suit: String) -> Card {
        Card { rank, suit }
    }
    fn to_string(&self) -> String {
        format!("{} of {}", self.rank, self.suit)
    }

    fn get_card_value(&self) -> u32 {
        match self.rank.as_str() {
            "Ace" => 14,
            "King" => 13,
            "Queen" => 12,
            "Jack" => 11,
            rank => rank.parse().unwrap_or(0),
        }
    }
}

pub struct Player {
    name: String,
    score: u32,
    keypair: Keypair,
    drawn_cards: Vec<(Card, VRFInOut, VRFProof)>,
}

impl Player {
    pub fn new(name: String) -> Self {
        let mut rng = OsRng;
        let keypair: Keypair = Keypair::generate_with(&mut rng);
        Player {
            name,
            score: 0,
            keypair,
            drawn_cards: vec![],
        }


    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_score(&self) -> u32 {
        self.score
    }

    pub fn set_score(&mut self, score: u32) {
        self.score = score;
    }

    pub fn verify_card(&self, card: &Card) -> bool {
        // Find the drawn card in the player's vector of drawn cards
        if let Some((_, output, proof)) = self.drawn_cards.iter().find(|(c, _, _)| c == card) {
            // Convert the card to a string and then to bytes
            let card_bytes = card.to_string().into_bytes();
            // Create a signing context
            let t = signing_context(self.get_name().as_bytes()).bytes(&card_bytes);
            // Verify the card
            self.keypair.public.vrf_verify(t, &output.to_preout(), proof).is_ok()
        } else {
            false
        }
    }

    pub fn draw_card(&mut self, deck: &Vec<Card>) -> Card {
        let mut rng = OsRng;

        let card_index = rng.gen_range(0..deck.len());
        let card = deck[card_index].clone();
        // Convert the card to a string and then to bytes
        let card_bytes = card.to_string().into_bytes();
        // Create a signing context
        let mut t = signing_context(self.get_name().as_bytes()).bytes(&card_bytes);
        // Use the keypair to sign the card bytes
        let (output, proof, _) = self.keypair.vrf_sign(&mut t);

        self.drawn_cards.push((card.clone(), output.clone(), proof.clone()));

        card
    }
}

fn main() {
    let mut player_alice = Player::new("Alice".to_string());
    let mut player_bob = Player::new("Bob".to_string());
    let cards_deck = generate_deck();

    let rounds = 1000; // Replace with the desired number of rounds
    let mut tie_score = 0;
    for _ in 0..rounds {
        let card_alice = player_alice.draw_card(&cards_deck);
        let card_bob= player_bob.draw_card(&cards_deck);


        let winner = compare_cards(&card_alice, &card_bob);

        //check if the card is valid


        match winner.as_str() {
            "Alice" => {
                let is_card_alice_valid = player_alice.verify_card(&card_alice);
                if is_card_alice_valid {
                    player_alice.set_score(player_alice.get_score() + 1);
                }else {
                    println!("Alice's card is invalid");
                }

            }
            "Bob" => {
                let is_card_bob_valid = player_bob.verify_card(&card_bob);
                if is_card_bob_valid {
                    player_bob.set_score(player_bob.get_score() + 1);
                } else {
                    println!("Bob's card is invalid");
                }

            }
            "Tie" => tie_score += 1,
            _ => println!("Error"),
        }
    }

    // who gets higher score wins
    if player_alice.get_score() > player_bob.get_score() {
        println!("Alice wins the game with a score of {:.2}", (player_alice.get_score() as f64 /rounds as f64) * 100.0);
    } else if player_bob.get_score() > player_alice.get_score() {
        println!("Bob wins the game with a score of {:.2}%", (player_bob.get_score() as f64 / rounds as f64) * 100.0);
    } else {
        println!("It's a tie!");
    }
    println!("Number of ties: {:.2}%", (tie_score as f64)/rounds as f64 * 100.0);


}


fn compare_cards(card_alice: &Card, card_bob: &Card) -> String {
    let value_alice = card_alice.get_card_value();
    let value_bob = card_bob.get_card_value();

    if value_alice > value_bob {
        "Alice".to_string()
    } else if value_bob > value_alice {
        "Bob".to_string()
    } else {
        "Tie".to_string()
    }
}
fn generate_deck() -> Vec<Card> {
    let suits = vec!["Spades", "Hearts", "Diamonds", "Clubs"];
    let ranks = vec![
        "Ace", "2", "3", "4", "5", "6", "7", "8", "9", "10", "Jack", "Queen", "King",
    ];

    let mut deck: Vec<Card> = Vec::new();

    for suit in &suits {
        for rank in &ranks {
            deck.push(Card::new(rank.to_string(), suit.to_string()));
        }
    }
    deck
}
