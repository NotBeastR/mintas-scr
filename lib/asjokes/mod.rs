use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct AsJokesModule;
impl AsJokesModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "joke" => Self::joke(args),
            "pun" => Self::pun(args),
            "fortune" => Self::fortune(args),
            "quote" => Self::quote(args),
            "fact" => Self::fact(args),
            "roast" => Self::roast(args),
            "compliment" => Self::compliment(args),
            "excuse" => Self::excuse(args),
            "wisdom" => Self::wisdom(args),
            "ascii_art" => Self::ascii_art(args),
            "cowsay" => Self::cowsay(args),
            "magic8ball" => Self::magic8ball(args),
            "dice" => Self::dice(args),
            "coin" => Self::coin(args),
            "rps" => Self::rps(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown asjokes function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn random_index(max: usize) -> usize {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        (nanos as usize) % max
    }
    fn joke(_args: &[Value]) -> MintasResult<Value> {
        let jokes = vec![
            "Why do programmers prefer dark mode? Because light attracts bugs!",
            "A SQL query walks into a bar, walks up to two tables and asks... 'Can I join you?'",
            "Why do Java developers wear glasses? Because they can't C#!",
            "There are only 10 types of people: those who understand binary and those who don't.",
            "Why was the JavaScript developer sad? Because he didn't Node how to Express himself!",
            "A programmer's wife tells him: 'Go to the store and buy a loaf of bread. If they have eggs, buy a dozen.' He comes home with 12 loaves.",
            "Why do programmers hate nature? It has too many bugs!",
            "What's a programmer's favorite hangout place? Foo Bar!",
            "Why did the developer go broke? Because he used up all his cache!",
            "How many programmers does it take to change a light bulb? None, that's a hardware problem!",
        ];
        Ok(Value::String(jokes[Self::random_index(jokes.len())].to_string()))
    }
    fn pun(_args: &[Value]) -> MintasResult<Value> {
        let puns = vec![
            "I'm reading a book about anti-gravity. It's impossible to put down!",
            "I used to hate facial hair, but then it grew on me.",
            "I'm on a seafood diet. I see food and I eat it!",
            "Time flies like an arrow. Fruit flies like a banana.",
            "I told my wife she was drawing her eyebrows too high. She looked surprised.",
        ];
        Ok(Value::String(puns[Self::random_index(puns.len())].to_string()))
    }
    fn fortune(_args: &[Value]) -> MintasResult<Value> {
        let fortunes = vec![
            "Your code will compile on the first try today!",
            "A great debugging session awaits you.",
            "You will find the missing semicolon.",
            "Stack Overflow will have the exact answer you need.",
            "Your pull request will be approved without changes.",
            "Today's bug will become tomorrow's feature.",
            "The documentation will actually be helpful today.",
            "Your tests will all pass... eventually.",
        ];
        Ok(Value::String(fortunes[Self::random_index(fortunes.len())].to_string()))
    }
    fn quote(_args: &[Value]) -> MintasResult<Value> {
        let quotes = vec![
            "\"Talk is cheap. Show me the code.\" - Linus Torvalds",
            "\"Programs must be written for people to read.\" - Harold Abelson",
            "\"Any fool can write code that a computer can understand.\" - Martin Fowler",
            "\"First, solve the problem. Then, write the code.\" - John Johnson",
            "\"Code is like humor. When you have to explain it, it's bad.\" - Cory House",
            "\"Simplicity is the soul of efficiency.\" - Austin Freeman",
            "\"Make it work, make it right, make it fast.\" - Kent Beck",
        ];
        Ok(Value::String(quotes[Self::random_index(quotes.len())].to_string()))
    }
    fn fact(_args: &[Value]) -> MintasResult<Value> {
        let facts = vec![
            "The first computer bug was an actual bug - a moth found in Harvard's Mark II computer in 1947.",
            "The first programmer was Ada Lovelace, who wrote algorithms for Charles Babbage's Analytical Engine.",
            "The name 'Python' comes from Monty Python, not the snake.",
            "JavaScript was created in just 10 days by Brendan Eich.",
            "The first computer virus was created in 1983 and was called 'Elk Cloner'.",
            "Git was created by Linus Torvalds in just 2 weeks.",
            "The @ symbol was almost removed from keyboards before email was invented.",
        ];
        Ok(Value::String(facts[Self::random_index(facts.len())].to_string()))
    }
    fn roast(_args: &[Value]) -> MintasResult<Value> {
        let roasts = vec![
            "Your code is so bad, even the compiler gave up and went home.",
            "I've seen better error handling in a 'Hello World' program.",
            "Your variable names are so cryptic, even you won't understand them tomorrow.",
            "That's not spaghetti code, that's a whole Italian restaurant.",
            "Your code comments are like your dating life - non-existent.",
        ];
        Ok(Value::String(roasts[Self::random_index(roasts.len())].to_string()))
    }
    fn compliment(_args: &[Value]) -> MintasResult<Value> {
        let compliments = vec![
            "Your code is cleaner than a freshly formatted drive!",
            "You write documentation that people actually want to read!",
            "Your commit messages tell a beautiful story.",
            "Your functions are so well-named, they're self-documenting!",
            "You're the kind of developer who makes code reviews enjoyable.",
            "Your error handling is chef's kiss!",
        ];
        Ok(Value::String(compliments[Self::random_index(compliments.len())].to_string()))
    }
    fn excuse(_args: &[Value]) -> MintasResult<Value> {
        let excuses = vec![
            "It works on my machine!",
            "That's not a bug, it's a feature.",
            "It must be a caching issue.",
            "The requirements changed.",
            "It was working yesterday!",
            "Someone must have changed something.",
            "The tests passed locally.",
            "It's a known issue, we're tracking it.",
            "Have you tried clearing your cache?",
        ];
        Ok(Value::String(excuses[Self::random_index(excuses.len())].to_string()))
    }
    fn wisdom(_args: &[Value]) -> MintasResult<Value> {
        let wisdom = vec![
            "A wise programmer once said: 'Delete code, not comments.'",
            "Remember: Weeks of coding can save you hours of planning.",
            "The best code is no code at all.",
            "When in doubt, add more logging.",
            "Premature optimization is the root of all evil.",
            "Always code as if the person maintaining your code is a violent psychopath who knows where you live.",
        ];
        Ok(Value::String(wisdom[Self::random_index(wisdom.len())].to_string()))
    }
    fn ascii_art(args: &[Value]) -> MintasResult<Value> {
        let art_type = if !args.is_empty() {
            match &args[0] {
                Value::String(s) => s.to_lowercase(),
                _ => "mintas".to_string(),
            }
        } else {
            "mintas".to_string()
        };
        let art = match art_type.as_str() {
            "cat" => r#"
  /\_/\  
 ( o.o ) 
  > ^ <
"#,
            "dog" => r#"
  / \__
 (    @\___
 /         O
/   (_____/
/_____/   U
"#,
            "heart" => r#"
  ♥♥   ♥♥
 ♥   ♥   ♥
  ♥     ♥
   ♥   ♥
    ♥ ♥
     ♥
"#,
            _ => r#"
  __  __ _       _            
 |  \/  (_)_ __ | |_ __ _ ___ 
 | |\/| | | '_ \| __/ _` / __|
 | |  | | | | | | || (_| \__ \
 |_|  |_|_|_| |_|\__\__,_|___/
"#,
        };
        Ok(Value::String(art.to_string()))
    }
    fn cowsay(args: &[Value]) -> MintasResult<Value> {
        let message = if !args.is_empty() {
            match &args[0] {
                Value::String(s) => s.clone(),
                _ => "Moo!".to_string(),
            }
        } else {
            "Moo!".to_string()
        };
        let border = "-".repeat(message.len() + 2);
        let cow = format!(r#"
 {}
< {} >
 {}
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
"#, border, message, border);
        Ok(Value::String(cow))
    }
    fn magic8ball(_args: &[Value]) -> MintasResult<Value> {
        let answers = vec![
            "It is certain.", "It is decidedly so.", "Without a doubt.",
            "Yes definitely.", "You may rely on it.", "As I see it, yes.",
            "Most likely.", "Outlook good.", "Yes.", "Signs point to yes.",
            "Reply hazy, try again.", "Ask again later.", "Better not tell you now.",
            "Cannot predict now.", "Concentrate and ask again.",
            "Don't count on it.", "My reply is no.", "My sources say no.",
            "Outlook not so good.", "Very doubtful.",
        ];
        Ok(Value::String(answers[Self::random_index(answers.len())].to_string()))
    }
    fn dice(args: &[Value]) -> MintasResult<Value> {
        let sides = if !args.is_empty() {
            match &args[0] {
                Value::Number(n) => *n as usize,
                _ => 6,
            }
        } else {
            6
        };
        let result = (Self::random_index(sides) + 1) as f64;
        Ok(Value::Number(result))
    }
    fn coin(_args: &[Value]) -> MintasResult<Value> {
        let result = if Self::random_index(2) == 0 { "heads" } else { "tails" };
        Ok(Value::String(result.to_string()))
    }
    fn rps(args: &[Value]) -> MintasResult<Value> {
        let choices = ["rock", "paper", "scissors"];
        let computer = choices[Self::random_index(3)];
        let player = if !args.is_empty() {
            match &args[0] {
                Value::String(s) => s.to_lowercase(),
                _ => return Ok(Value::String(format!("Computer chose: {}", computer))),
            }
        } else {
            return Ok(Value::String(format!("Computer chose: {}", computer)));
        };
        let result = match (player.as_str(), computer) {
            (p, c) if p == c => "It's a tie!",
            ("rock", "scissors") | ("paper", "rock") | ("scissors", "paper") => "You win!",
            _ => "Computer wins!",
        };
        let mut table = HashMap::new();
        table.insert("player".to_string(), Value::String(player));
        table.insert("computer".to_string(), Value::String(computer.to_string()));
        table.insert("result".to_string(), Value::String(result.to_string()));
        Ok(Value::Table(table))
    }
}