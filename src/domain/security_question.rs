//! Security questions for password recovery

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::crypto::{SecureString, hash_security_answer, verify_security_answer};
use crate::utils::error::Result;
use crate::utils::mask::partial_reveal;

/// A security question with hashed answer
#[derive(Clone, Serialize, Deserialize)]
pub struct SecurityQuestion {
    /// The question text
    pub question: String,
    /// Hashed answer (for verification)
    pub answer_hash: Vec<u8>,
    /// Salt used for hashing
    pub salt: [u8; 32],
}

impl fmt::Debug for SecurityQuestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecurityQuestion")
            .field("question", &"[REDACTED]")
            .field("answer_hash", &"[REDACTED]")
            .field("salt", &"[REDACTED]")
            .finish()
    }
}

impl SecurityQuestion {
    /// Create a new security question with the answer hashed
    pub fn new(question: impl Into<String>, answer: &SecureString) -> Result<Self> {
        let (answer_hash, salt) = hash_security_answer(answer)?;
        Ok(Self {
            question: question.into(),
            answer_hash,
            salt,
        })
    }

    /// Verify if an answer is correct
    pub fn verify(&self, answer: &SecureString) -> Result<bool> {
        verify_security_answer(answer, &self.answer_hash, &self.salt)
    }
}

/// Configuration for progressive password recovery
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Percentage of characters revealed per correct answer
    /// Index 0 = after 1st correct answer
    /// Index 1 = after 2nd correct answer
    /// Index 2 = after 3rd correct answer
    pub reveal_percentages: [f32; 3],
    /// Maximum number of recovery attempts
    pub max_attempts: u32,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            reveal_percentages: [0.3, 0.5, 0.8],
            max_attempts: 5,
        }
    }
}

impl RecoveryConfig {
    /// Get reveal percentage for a given number of correct answers
    pub fn get_reveal_percentage(&self, correct_count: usize) -> f32 {
        match correct_count {
            0 => 0.0,
            1 => self.reveal_percentages[0],
            2 => self.reveal_percentages[1],
            _ => self.reveal_percentages[2],
        }
    }
}

/// State for an ongoing password recovery attempt
#[derive(Debug)]
pub struct RecoveryState {
    /// The questions to answer
    pub questions: Vec<SecurityQuestion>,
    /// Which questions have been answered correctly
    pub correct_answers: Vec<bool>,
    /// Number of failed attempts
    pub failed_attempts: u32,
    /// Recovery configuration
    pub config: RecoveryConfig,
    /// Seed for consistent partial reveal
    pub reveal_seed: u64,
}

impl RecoveryState {
    /// Create a new recovery state
    pub fn new(questions: Vec<SecurityQuestion>, reveal_seed: u64) -> Self {
        let correct_answers = vec![false; questions.len()];
        Self {
            questions,
            correct_answers,
            failed_attempts: 0,
            config: RecoveryConfig::default(),
            reveal_seed,
        }
    }

    /// Attempt to answer a question
    pub fn attempt_answer(&mut self, question_index: usize, answer: &SecureString) -> Result<bool> {
        if question_index >= self.questions.len() {
            return Ok(false);
        }

        let is_correct = self.questions[question_index].verify(answer)?;

        if is_correct {
            self.correct_answers[question_index] = true;
        } else {
            self.failed_attempts += 1;
        }

        Ok(is_correct)
    }

    /// Get the number of correctly answered questions
    pub fn correct_count(&self) -> usize {
        self.correct_answers.iter().filter(|&&x| x).count()
    }

    /// Check if max attempts exceeded
    pub fn is_locked_out(&self) -> bool {
        self.failed_attempts >= self.config.max_attempts
    }

    /// Get the partially revealed password based on correct answers
    pub fn reveal_password(&self, password: &str) -> String {
        let percentage = self.config.get_reveal_percentage(self.correct_count());
        if percentage == 0.0 {
            return "•".repeat(password.len().min(16));
        }
        partial_reveal(password, percentage, self.reveal_seed)
    }

    /// Check if all questions answered correctly
    pub fn is_complete(&self) -> bool {
        self.correct_answers.iter().all(|&x| x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_question_verify() {
        let answer = SecureString::from_str("fluffy");
        let question = SecurityQuestion::new("What is your pet's name?", &answer).unwrap();

        assert!(question.verify(&SecureString::from_str("fluffy")).unwrap());
        assert!(!question.verify(&SecureString::from_str("spot")).unwrap());
    }

    #[test]
    fn test_recovery_config_percentages() {
        let config = RecoveryConfig::default();

        assert_eq!(config.get_reveal_percentage(0), 0.0);
        assert_eq!(config.get_reveal_percentage(1), 0.3);
        assert_eq!(config.get_reveal_percentage(2), 0.5);
        assert_eq!(config.get_reveal_percentage(3), 0.8);
    }

    #[test]
    fn test_recovery_state() {
        let q1 = SecurityQuestion::new("Q1?", &SecureString::from_str("a1")).unwrap();
        let q2 = SecurityQuestion::new("Q2?", &SecureString::from_str("a2")).unwrap();

        let mut state = RecoveryState::new(vec![q1, q2], 12345);

        assert_eq!(state.correct_count(), 0);
        assert!(!state.is_complete());

        // Answer first question correctly
        assert!(
            state
                .attempt_answer(0, &SecureString::from_str("a1"))
                .unwrap()
        );
        assert_eq!(state.correct_count(), 1);

        // Wrong answer
        assert!(
            !state
                .attempt_answer(1, &SecureString::from_str("wrong"))
                .unwrap()
        );
        assert_eq!(state.failed_attempts, 1);

        // Correct second answer
        assert!(
            state
                .attempt_answer(1, &SecureString::from_str("a2"))
                .unwrap()
        );
        assert!(state.is_complete());
    }

    #[test]
    fn test_recovery_lockout() {
        let q = SecurityQuestion::new("Q?", &SecureString::from_str("answer")).unwrap();
        let mut state = RecoveryState::new(vec![q], 12345);

        for _ in 0..5 {
            let _ = state.attempt_answer(0, &SecureString::from_str("wrong"));
        }

        assert!(state.is_locked_out());
    }

    #[test]
    fn test_partial_reveal() {
        let q = SecurityQuestion::new("Q?", &SecureString::from_str("a")).unwrap();
        let mut state = RecoveryState::new(vec![q], 12345);

        let password = "kucinghitam";

        // No answers - fully masked (max 16 chars)
        let revealed = state.reveal_password(password);
        assert!(!revealed.contains('k'));
        assert!(revealed.chars().all(|c| c == '•'));

        // One correct answer - partial reveal
        let _ = state.attempt_answer(0, &SecureString::from_str("a"));
        let revealed = state.reveal_password(password);
        // Should have same length as password now (partial reveal)
        assert_eq!(revealed.chars().count(), password.chars().count());
        // Should have some revealed characters
        assert!(revealed.chars().any(|c| c != '•'));
    }
}
