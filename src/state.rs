use regex::Regex;

pub struct Match<'a> {
  pub x: i32,
  pub y: i32,
  pub text: &'a str,
  pub hint: Option<String>
}

impl<'a> PartialEq for Match<'a> {
  fn eq(&self, other: &Match) -> bool {
    self.x == other.x && self.y == other.y
  }
}

pub struct State<'a> {
  pub lines: Vec<&'a str>,
  alphabet: &'a str,
  pub skip: usize,
}

impl<'a> State<'a> {
  pub fn new(lines: Vec<&'a str>, alphabet: &'a str) -> State<'a> {
    State{
      lines: lines,
      alphabet: alphabet,
      skip: 0
    }
  }

  pub fn matches(&self) -> Vec<Match<'a>> {
    let mut matches = Vec::new();
    let mut patterns = Vec::new();

    // TODO: Improve pattern preference
    patterns.push(Regex::new("((^|^\\.|[[:space:]]|[[:space:]]\\.|[[:space:]]\\.\\.|^\\.\\.)[[:alnum:]~_-]*/\\[\\][[:alnum:]_.#$%&+=/@-]+)").unwrap()); // Paths
    patterns.push(Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap()); // Uid
    patterns.push(Regex::new(r"[0-9a-f]{7,40}").unwrap()); // Sha id
    // patterns.push(Regex::new(r"[0-9]{4,}").unwrap()); // Process or ports
    patterns.push(Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap()); // Ip address
    patterns.push(Regex::new(r"((https?://|git@|git://|ssh://|ftp://|file:///)[\w?=%/_.:,;~@!#$&()*+-]*)").unwrap()); // Urls
    patterns.push(Regex::new(r"(0x[0-9a-fA-F]+)").unwrap()); // Address

    for (index, line) in self.lines.iter().enumerate() {
      for pattern in patterns.iter() {
        for mat in pattern.find_iter(line) {
          matches.push(Match{
            x: mat.start() as i32,
            y: index as i32,
            text: &line[mat.start()..mat.end()],
            hint: None
          });
        }
      }
    }

    let alphabet = super::alphabets::get_alphabet(self.alphabet);
    let mut hints = alphabet.hints(matches.len());

    for mat in &mut matches {
      mat.hint = Some(hints.pop().unwrap().to_string().clone())
    }

    return matches;
  }

  pub fn prev(&mut self) {
    if self.skip > 0 {
      self.skip = self.skip - 1;
    }
  }

  pub fn next(&mut self) {
    self.skip = self.skip + 1;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn match_lines(output: &str) -> Vec<Match> {
    let lines = output.split("\n").collect::<Vec<&str>>();
    let state = State::new(lines, "abcd");

    state.matches()
  }

  #[test]
  fn match_paths () {
    let output = "Lorem /tmp/foo/bar lorem\n Lorem /var/log/bootstrap.log lorem /var/log/kern.log lorem";

    assert_ne!(match_lines(output).len(), 2); // FIXME regex priority
  }

  #[test]
  fn match_uids () {
    let output = "Lorem ipsum 123e4567-e89b-12d3-a456-426655440000 lorem\n Lorem lorem lorem";

    assert_ne!(match_lines(output).len(), 1); // FIXME regex priority
  }

  #[test]
  fn match_shas () {
    let output = "Lorem fd70b5695 5246ddf f924213 lorem\n Lorem 973113963b491874ab2e372ee60d4b4cb75f717c lorem";

    assert_eq!(match_lines(output).len(), 4);
  }

  #[test]
  fn match_ips () {
    let output = "Lorem ipsum 127.0.0.1 lorem\n Lorem 255.255.10.255 lorem 127.0.0.1 lorem";

    assert_eq!(match_lines(output).len(), 3);
  }

  #[test]
  fn match_urls () {
    let output = "Lorem ipsum https://www.rust-lang.org/tools lorem\n Lorem https://crates.io lorem https://github.io lorem ssh://github.io";

    assert_eq!(match_lines(output).len(), 4);
  }

  #[test]
  fn match_addresses () {
    let output = "Lorem 0xfd70b5695 0x5246ddf lorem\n Lorem 0x973113 lorem";

    assert_ne!(match_lines(output).len(), 3); // FIXME regex priority
  }
}
