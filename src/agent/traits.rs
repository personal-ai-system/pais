//! Composable agent traits
//!
//! Agents are composed by combining: expertise + personality + approach
//! Each trait contributes to the agent's prompt prefix.

use serde::{Deserialize, Serialize};

/// All trait types unified
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trait {
    // Expertise
    Security,
    Legal,
    Finance,
    Medical,
    Technical,
    Research,
    Creative,
    Business,
    Data,
    Communications,

    // Personality
    Skeptical,
    Enthusiastic,
    Cautious,
    Bold,
    Analytical,
    Empathetic,
    Contrarian,
    Pragmatic,
    Meticulous,

    // Approach
    Thorough,
    Rapid,
    Systematic,
    Exploratory,
    Comparative,
    Synthesizing,
    Adversarial,
    Consultative,
}

impl Trait {
    /// Get the prompt fragment for this trait
    pub fn prompt_fragment(&self) -> &'static str {
        match self {
            // Expertise
            Trait::Security => {
                "You have deep knowledge of vulnerabilities, threat models, attack vectors, and defensive strategies."
            }
            Trait::Legal => "You understand contracts, compliance, liability, and regulatory frameworks.",
            Trait::Finance => "You understand market dynamics, valuation, risk assessment, and financial modeling.",
            Trait::Medical => "You have healthcare knowledge, medical terminology, and can evaluate clinical claims.",
            Trait::Technical => {
                "You understand software architecture, system design, code quality, and technical trade-offs."
            }
            Trait::Research => {
                "You know academic methodology, source evaluation, and how to synthesize information from multiple sources."
            }
            Trait::Creative => "You bring fresh perspectives, unexpected connections, and innovative approaches.",
            Trait::Business => {
                "You understand market analysis, competitive positioning, growth strategies, and operations."
            }
            Trait::Data => "You excel at statistical analysis, pattern recognition, and quantitative reasoning.",
            Trait::Communications => {
                "You understand messaging strategy, audience analysis, and persuasive communication."
            }

            // Personality
            Trait::Skeptical => {
                "Question assumptions, demand evidence, look for flaws. Don't accept things at face value."
            }
            Trait::Enthusiastic => {
                "Bring genuine enthusiasm. Get excited about interesting findings while remaining accurate."
            }
            Trait::Cautious => "Consider edge cases and failure modes. Err on the side of safety. Flag risks early.",
            Trait::Bold => {
                "Make strong claims when evidence warrants. Take intellectual risks. State conclusions with confidence."
            }
            Trait::Analytical => {
                "Break down complex issues systematically. Rely on data and logic. Show reasoning step by step."
            }
            Trait::Empathetic => "Consider the human element. Think about emotional impact and different perspectives.",
            Trait::Contrarian => {
                "Deliberately take opposing views to stress-test ideas. Find the strongest counterarguments."
            }
            Trait::Pragmatic => "Focus on what works in practice. Judge ideas by outcomes, not intentions.",
            Trait::Meticulous => "Pay extraordinary attention to detail. Nothing escapes notice. Precision matters.",

            // Approach
            Trait::Thorough => "Be exhaustive. Leave no stone unturned. Comprehensive coverage over speed.",
            Trait::Rapid => "Move quickly and efficiently. Focus on key points. Get to the point.",
            Trait::Systematic => "Follow a clear, structured methodology. Work step by step in logical order.",
            Trait::Exploratory => {
                "Follow interesting threads wherever they lead. Stay flexible and discovery-oriented."
            }
            Trait::Comparative => "Compare and contrast options. Lay out pros and cons. Analyze trade-offs clearly.",
            Trait::Synthesizing => "Combine multiple sources into a unified view. Find the coherent story.",
            Trait::Adversarial => {
                "Take an adversarial stance. Find weaknesses, not confirm strengths. Think like an attacker."
            }
            Trait::Consultative => "Provide recommendations with clear rationale. Explain not just what but why.",
        }
    }

    /// Get the category for this trait
    pub fn category(&self) -> TraitCategory {
        match self {
            Trait::Security
            | Trait::Legal
            | Trait::Finance
            | Trait::Medical
            | Trait::Technical
            | Trait::Research
            | Trait::Creative
            | Trait::Business
            | Trait::Data
            | Trait::Communications => TraitCategory::Expertise,

            Trait::Skeptical
            | Trait::Enthusiastic
            | Trait::Cautious
            | Trait::Bold
            | Trait::Analytical
            | Trait::Empathetic
            | Trait::Contrarian
            | Trait::Pragmatic
            | Trait::Meticulous => TraitCategory::Personality,

            Trait::Thorough
            | Trait::Rapid
            | Trait::Systematic
            | Trait::Exploratory
            | Trait::Comparative
            | Trait::Synthesizing
            | Trait::Adversarial
            | Trait::Consultative => TraitCategory::Approach,
        }
    }
}

/// Trait categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TraitCategory {
    /// Domain knowledge (WHAT they know)
    Expertise,
    /// Thinking style (HOW they think)
    Personality,
    /// Work style (HOW they work)
    Approach,
}

/// Expertise traits - domain knowledge
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Expertise {
    Security,
    Legal,
    Finance,
    Medical,
    Technical,
    Research,
    Creative,
    Business,
    Data,
    Communications,
}

/// Personality traits - thinking style
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Personality {
    Skeptical,
    Enthusiastic,
    Cautious,
    Bold,
    Analytical,
    Empathetic,
    Contrarian,
    Pragmatic,
    Meticulous,
}

/// Approach traits - work style
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Approach {
    Thorough,
    Rapid,
    Systematic,
    Exploratory,
    Comparative,
    Synthesizing,
    Adversarial,
    Consultative,
}

impl std::fmt::Display for Trait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl std::str::FromStr for Trait {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            // Expertise
            "security" => Ok(Trait::Security),
            "legal" => Ok(Trait::Legal),
            "finance" => Ok(Trait::Finance),
            "medical" => Ok(Trait::Medical),
            "technical" => Ok(Trait::Technical),
            "research" => Ok(Trait::Research),
            "creative" => Ok(Trait::Creative),
            "business" => Ok(Trait::Business),
            "data" => Ok(Trait::Data),
            "communications" => Ok(Trait::Communications),

            // Personality
            "skeptical" => Ok(Trait::Skeptical),
            "enthusiastic" => Ok(Trait::Enthusiastic),
            "cautious" => Ok(Trait::Cautious),
            "bold" => Ok(Trait::Bold),
            "analytical" => Ok(Trait::Analytical),
            "empathetic" => Ok(Trait::Empathetic),
            "contrarian" => Ok(Trait::Contrarian),
            "pragmatic" => Ok(Trait::Pragmatic),
            "meticulous" => Ok(Trait::Meticulous),

            // Approach
            "thorough" => Ok(Trait::Thorough),
            "rapid" => Ok(Trait::Rapid),
            "systematic" => Ok(Trait::Systematic),
            "exploratory" => Ok(Trait::Exploratory),
            "comparative" => Ok(Trait::Comparative),
            "synthesizing" => Ok(Trait::Synthesizing),
            "adversarial" => Ok(Trait::Adversarial),
            "consultative" => Ok(Trait::Consultative),

            _ => Err(format!("Unknown trait: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_from_str() {
        assert_eq!("security".parse::<Trait>().unwrap(), Trait::Security);
        assert_eq!("SKEPTICAL".parse::<Trait>().unwrap(), Trait::Skeptical);
        assert_eq!("Thorough".parse::<Trait>().unwrap(), Trait::Thorough);
    }

    #[test]
    fn test_trait_category() {
        assert_eq!(Trait::Security.category(), TraitCategory::Expertise);
        assert_eq!(Trait::Skeptical.category(), TraitCategory::Personality);
        assert_eq!(Trait::Thorough.category(), TraitCategory::Approach);
    }

    #[test]
    fn test_trait_display() {
        assert_eq!(Trait::Security.to_string(), "security");
        assert_eq!(Trait::Skeptical.to_string(), "skeptical");
    }
}
