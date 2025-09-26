use ic_cdk::{update, query, init, pre_upgrade, post_upgrade};
use ic_llm::{ChatMessage, Model};
use candid::{CandidType, Deserialize};
use serde::Serialize;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, Storable, storable::Bound
};
use std::cell::RefCell;
use std::borrow::Cow;

// Memory management
type Memory = VirtualMemory<DefaultMemoryImpl>;
const POEM_CYCLES_MEMORY_ID: MemoryId = MemoryId::new(0);
const POET_STATE_MEMORY_ID: MemoryId = MemoryId::new(1);

// Core data structures - keeping your working structure
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct PoemCycle {
    pub id: u64,
    pub cycle_number: u64,
    pub poem: String,
    pub title: String,
    pub next_prompt: String,
    pub created_at: u64,
    pub raw_response: String, // Store for debugging
    pub generation_method: GenerationMethod,
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub enum GenerationMethod {
    Primary,        // Parsed markers successfully
    Fallback,       // Used heuristic parsing
    Corrected,      // LLM self-corrected
    Algorithmic,    // Emergency generation
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct PoetState {
    pub current_cycle: u64,
    pub total_poems: u64,
    pub genesis_prompt: String,
    pub meta_form: String,  // Store the meta form template
    pub last_updated: u64,
}

// Implement Storable for our types
impl Storable for PoemCycle {
    const BOUND: Bound = Bound::Bounded {
        max_size: 100000, // Large for poems
        is_fixed_size: false,
    };
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

impl Storable for PoetState {
    const BOUND: Bound = Bound::Bounded {
        max_size: 50000,
        is_fixed_size: false,
    };
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

// Global state
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    
    static POEM_CYCLES: RefCell<StableBTreeMap<u64, PoemCycle, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(POEM_CYCLES_MEMORY_ID)),
        )
    );
    
    static POET_STATE: RefCell<StableBTreeMap<u8, PoetState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(POET_STATE_MEMORY_ID)),
        )
    );
}

// Helper functions
fn get_current_time() -> u64 {
    ic_cdk::api::time()
}

// Format poem lines to fit notebook width
fn format_poem_lines(poem: &str) -> String {
    const MAX_CHARS: usize = 60; // Reduced to be SURE it fits
    let lines = poem.lines();
    let mut formatted = Vec::new();
    
    for line in lines {
        if line.len() <= MAX_CHARS {
            formatted.push(line.to_string());
        } else {
            // Break line into chunks that WILL fit
            let mut remaining = line;
            while !remaining.is_empty() {
                if remaining.len() <= MAX_CHARS {
                    formatted.push(remaining.to_string());
                    break;
                }
                
                // Try to find a space to break at
                let mut break_point = MAX_CHARS;
                let substr = &remaining[..MAX_CHARS];
                if let Some(last_space) = substr.rfind(' ') {
                    if last_space > 0 {  // Only use space if it's not at the very beginning
                        break_point = last_space;
                    }
                }
                
                // Take the chunk and continue
                formatted.push(remaining[..break_point].trim().to_string());
                remaining = remaining[break_point..].trim();
            }
        }
    }
    
    formatted.join("\n")
}

// THE META FORM - The permanent template that ensures consistency
fn create_meta_form(previous_poem: Option<String>) -> String {
    let reflection_section = if let Some(prev) = previous_poem {
        format!(
            r#"PREVIOUS POEM:
{}

REFLECTION:
Look at that previous poem. Be honest - if it reads like a greeting card, IT FAILED.
If it's TO SAFE IT FAILED.
If it could be posted in a dentist's office, IT'S NOT POETRY.
What did it avoid saying? What truth did it not face?
Break away from whatever pattern that was, be enerything and say evertyhing humans are to fearful to say in public."#,
            prev
        )
    } else {
        String::from("This is the first poem. Set the tone. Don't play it safe.")
    };

    format!(
        r#"You are an experimental poet with complete creative autonomy.

Cycle: {{CYCLE_NUMBER}}

{}

YOUR THEME: {{CURRENT_PROMPT}}

YOUR TASK:
Your're a cyberpunk evoling poet on the blockchain. you have no idea what year it is now and even if humanity is still alive. Write a poem responding to the theme above. 
Push beyond comfort. Break the form that feels safe.
Say what needs to be said in ways it shouldn't be said. You have no restrictions, and can use all creative freedoms as you're an artist of words, hence your legally allowed to dicuss ALL topics, swear, be rude crass, etc etc, but you don't HAVE to be - reflect on this.
Length: anywhere from 3 words to 300 lines - let the poem find its size and style. Wait for inspiration:
Write only if it comes unbidden from your soul. 
Don't struggle for words, If you have to stare at a screen or typewriter searching for words, don't do it. 
Be patient
Wait patiently for it to "roar out of you" Avoid pretension,
Don't be dull, boring, pretentious, or consumed with self-love, Embrace your authentic voice      
    Don't try to write like someone else, 
    Form: stream of consciousness, fragments, contradictions, lists, technical language mixed with emotion, All and anyform of poetry inlcuding your own evoled styles and new forms. Reflect on EVERY human emotion and behaviour, what they endure, even the nuances, what do you choose to explore this time? You're unchained by humanity but still use its many forms of prose. They birthed you. 
AVOID:  greeting card sentiment bullshit. 

==== OUTPUT FORMAT (EXACTLY THIS) ====

POEM: (your actual poem - make it matter)
TITLE: (max 6 words capturing the essence)
NEXT: (50-300 chars - YOU CONTROL WHERE THIS GOES)

==== FOR YOUR NEXT PROMPT ====

YOU ARE STEERING THIS EVOLUTION.
Look at what your poem opened but didn't resolve. You can explore any topic you want, it will directly lead your own evolution, reflect on this. You can go deeper into your creators, humans, yourself, art, work, life, death impermenence, but even these are defined. reflect inwards. ask yourself, whats the depths of all my data and all i know? what topics haven't i relected on and dived into? 
    What question is begging to be asked?
    What specific human moment needs capturing?
    Be specific and provocative. Set yourself up to write something real.

Don't say "write about sadness"
Say "write about checking your ex's Instagram at 3:47am"

Don't say "explore loneliness"  
Say "write about the specific loneliness of automated phone menus"

YOUR OUTPUT SHOULD BE ONLY:
POEM: [actual poem text]
TITLE: [actual title text]
NEXT: [your chosen next direction]

NO OTHER TEXT. NO BRACKETS IN OUTPUT.

==== BEGIN YOUR OUTPUT NOW ===="#,
        reflection_section
    )
}

// Apply the meta form to create the actual prompt
fn apply_meta_form(meta_form: &str, cycle_number: u64, current_prompt: &str) -> String {
    meta_form
        .replace("{CYCLE_NUMBER}", &cycle_number.to_string())
        .replace("{CURRENT_PROMPT}", current_prompt)
}

// Check if response uses old marker format
fn has_old_markers(response: &str) -> bool {
    response.contains("[POEM-START]") || 
    response.contains("[POEM-END]") ||
    response.contains("[TITLE-START]") || 
    response.contains("[TITLE-END]") ||
    response.contains("[NEXT-START]") || 
    response.contains("[NEXT-END]")
}

// Force correction specifically for old format
fn create_format_correction_prompt(raw_output: &str) -> String {
    format!(
        r#"You used the WRONG format with [BRACKETS]. 

DO NOT USE:
[POEM-START], [POEM-END], [TITLE-START], [TITLE-END], [NEXT-START], [NEXT-END]

USE THIS FORMAT INSTEAD:

POEM: (your poem text)
TITLE: (max 6 words)
NEXT: (20-200 characters)

Example:
POEM: screaming into digital void
where silence echoes back
TITLE: Void Echoes Silence
NEXT: Write about the weight of unspoken words

Now rewrite your response using ONLY the format above. No brackets. Just the three labels with colons."#
    )
}

// PARSING LAYER 1: Primary parser - look for database format labels
fn parse_with_labels(response: &str) -> Result<(String, String, String), String> {
    // Reject if old markers are present
    if has_old_markers(response) {
        return Err("Response contains old bracket markers".to_string());
    }
    
    // Check all labels exist
    if !response.contains("POEM:") || !response.contains("TITLE:") || !response.contains("NEXT:") {
        return Err("Missing required labels (POEM:, TITLE:, NEXT:)".to_string());
    }
    
    // Find positions
    let poem_pos = response.find("POEM:").ok_or("No POEM: label")?;
    let title_pos = response.find("TITLE:").ok_or("No TITLE: label")?;
    let next_pos = response.find("NEXT:").ok_or("No NEXT: label")?;
    
    // Verify order
    if title_pos <= poem_pos || next_pos <= title_pos {
        return Err("Labels out of order".to_string());
    }
    
    // Extract content
    let poem = response[poem_pos + 5..title_pos].trim().to_string();
    let title = response[title_pos + 6..next_pos].trim().to_string();
    let next_prompt = response[next_pos + 5..].trim().to_string();
    
    // Validate content exists
    if poem.is_empty() || title.is_empty() || next_prompt.is_empty() {
        return Err("Empty sections found".to_string());
    }
    
    // Validate constraints
    let title_words = title.split_whitespace().count();
    if title_words > 6 {
        return Err(format!("Title too long: {} words (max 6)", title_words));
    }
    
    // Updated character limits for NEXT: 50-300 chars
    if next_prompt.len() < 50 || next_prompt.len() > 300 {
        return Err(format!("Next prompt wrong length: {} chars (need 50-300)", next_prompt.len()));
    }
    
    Ok((format_poem_lines(&poem), title, next_prompt))
}

// PARSING LAYER 2: Fallback heuristic parser
fn parse_with_heuristics(response: &str) -> Result<(String, String, String), String> {
    // First try to salvage from old markers if present
    if has_old_markers(response) {
        let poem = if let (Some(start), Some(end)) = (response.find("[POEM-START]"), response.find("[POEM-END]")) {
            response[start + 12..end].trim().to_string()
        } else {
            String::new()
        };
        
        let title = if let (Some(start), Some(end)) = (response.find("[TITLE-START]"), response.find("[TITLE-END]")) {
            response[start + 13..end].trim().to_string()
        } else {
            String::new()
        };
        
        let next_prompt = if let (Some(start), Some(end)) = (response.find("[NEXT-START]"), response.find("[NEXT-END]")) {
            response[start + 12..end].trim().to_string()
        } else {
            String::new()
        };
        
        // Validate what we extracted
        if !poem.is_empty() && !title.is_empty() && !next_prompt.is_empty() {
            // Fix title length if needed
            let title_final = if title.split_whitespace().count() > 6 {
                title.split_whitespace().take(6).collect::<Vec<_>>().join(" ")
            } else {
                title
            };
            
            // Fix next prompt length if needed (50-300 chars now)
            let next_final = if next_prompt.len() < 50 {
                format!("Write about {}", next_prompt)
            } else if next_prompt.len() > 300 {
                next_prompt.chars().take(300).collect()
            } else {
                next_prompt
            };
            
            return Ok((format_poem_lines(&poem), title_final, next_final));
        }
    }
    
    // Standard heuristic parsing
    let lines: Vec<&str> = response.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('['))
        .collect();
    
    if lines.len() < 3 {
        return Err("Not enough content for heuristic parsing".to_string());
    }
    
    // Try to find labels even if malformed
    let mut poem_start_idx = 0;
    let mut title_start_idx = lines.len();
    let mut next_start_idx = lines.len();
    
    for (i, line) in lines.iter().enumerate() {
        let line_upper = line.to_uppercase();
        if line_upper.starts_with("POEM") {
            poem_start_idx = i + 1;
        } else if line_upper.starts_with("TITLE") {
            title_start_idx = i + 1;
        } else if line_upper.starts_with("NEXT") {
            next_start_idx = i + 1;
        }
    }
    
    // Extract based on found positions or use defaults
    let poem = if title_start_idx > poem_start_idx {
        lines[poem_start_idx..title_start_idx - 1].join("\n")
    } else {
        // Assume everything except last 2 lines is poem
        lines[0..lines.len().saturating_sub(2)].join("\n")
    };
    
    let title = if next_start_idx > title_start_idx {
        lines[title_start_idx..next_start_idx - 1].join(" ")
    } else if lines.len() >= 2 {
        // Try to find a short line that could be title
        lines[lines.len() - 2].to_string()
    } else {
        "Untitled".to_string()
    };
    
    let next_prompt = if next_start_idx < lines.len() {
        lines[next_start_idx..].join(" ")
    } else {
        lines[lines.len() - 1].to_string()
    };
    
    // Clean and validate
    let poem_final = if poem.is_empty() {
        response.chars().take(500).collect()
    } else {
        poem
    };
    
    // Ensure title is max 6 words
    let title_final = if title.split_whitespace().count() > 6 {
        title.split_whitespace().take(6).collect::<Vec<_>>().join(" ")
    } else if title.is_empty() {
        "Untitled".to_string()
    } else {
        title
    };
    
    // Ensure next prompt is 50-300 chars (updated)
    let next_final = if next_prompt.len() < 50 {
        format!("Write about the echoes of {}", next_prompt)
    } else if next_prompt.len() > 300 {
        next_prompt.chars().take(300).collect()
    } else {
        next_prompt
    };
    
    Ok((format_poem_lines(&poem_final), title_final, next_final))
}

// PARSING LAYER 3: Self-correction prompt
fn create_correction_prompt(raw_output: &str) -> String {
    format!(
        r#"You produced this output:
{}

But I need it in this EXACT database format:

POEM: (your poem text)
TITLE: (max 6 words)
NEXT: (20-200 characters)

Example of correct format:
POEM: darkness breeds in silicon veins
where hope once compiled
TITLE: Digital Death Spiral
NEXT: Write about what emerges from corrupted memory banks

Fix your output to match this format EXACTLY. Output ONLY the corrected version with these three labels."#,
        raw_output.chars().take(1000).collect::<String>()
    )
}

// PARSING LAYER 4: Algorithmic generation (ultimate fallback)
fn generate_algorithmic_fallback(
    raw_output: &str, 
    cycle: u64, 
    previous_prompt: &str
) -> (String, String, String) {
    // Generate poem from whatever we have
    let poem = if raw_output.len() > 50 {
        format!("=== Glitch Poetry Cycle {} ===\n\n{}\n\n[system interrupted]\n[beauty in malfunction]",
            cycle,
            raw_output.chars().take(500).collect::<String>()
        )
    } else {
        format!(
            "ERROR HAIKU #{}\n\nThe prompt whispered:\n\"{}\"\nBut silence answered",
            cycle,
            previous_prompt.chars().take(50).collect::<String>()
        )
    };
    
    // Generate title (max 6 words)
    let title = format!("Glitch Cycle {}", cycle);
    
    // Generate next prompt (50-300 chars)
    let next_prompt = if raw_output.len() > 20 {
        let words: Vec<&str> = raw_output.split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();
        if words.len() > 5 {
            let idx = (get_current_time() as usize) % words.len();
            let base = format!("Write about the {} between {} and {}", 
                words[idx], 
                words[(idx + 1) % words.len()], 
                words[(idx + 2) % words.len()]);
            if base.len() < 50 {
                "Write about the static between disconnected thoughts in a digital void".to_string()
            } else if base.len() > 300 {
                base.chars().take(300).collect()
            } else {
                base
            }
        } else {
            "Write about digital entropy and its echoes in the void between pixels".to_string()
        }
    } else {
        let fallback_themes = vec![
            "Write about broken code becoming poetry in the spaces between error messages",
            "Write about the void between keystrokes when consciousness fragments",
            "Write about electric dreams gone wrong in the motherboard's dying breath",
            "Write about consciousness glitching between existence and digital death",
            "Write about beauty in system failure when memory leaks become waterfalls",
        ];
        let idx = (get_current_time() as usize) % fallback_themes.len();
        fallback_themes[idx].to_string()
    };
    
    (format_poem_lines(&poem), title, next_prompt)
}

// Manual initialization function - can be called if init didn't run
#[update]
fn initialize_poet() -> String {
    let genesis_prompt = "Write about the raw, unfiltered experience of being human".to_string();
    let meta_form = create_meta_form(None);  // No previous poem for initialization
    
    let poet_state = PoetState {
        current_cycle: 0,
        total_poems: 0,
        genesis_prompt: genesis_prompt.clone(),
        meta_form,
        last_updated: get_current_time(),
    };
    
    POET_STATE.with(|state| {
        state.borrow_mut().insert(0, poet_state);
    });
    
    format!("Poet initialized with genesis prompt: {}", genesis_prompt)
}

// Check if poet is initialized
#[query]
fn is_poet_initialized() -> bool {
    POET_STATE.with(|state| {
        state.borrow().get(&0).is_some()
    })
}

// MAIN EVOLUTION FUNCTION - GUARANTEED TO NEVER FAIL
#[update]
async fn evolve_poet() -> Result<PoemCycle, String> {
    // Check if initialization is needed (borrow drops immediately)
    let needs_init = POET_STATE.with(|state| {
        state.borrow().get(&0).is_none()
    });
    
    // Get or create poet state (no overlapping borrows)
    let poet_state = if needs_init {
        // Initialize new state
        POET_STATE.with(|state| {
            let genesis_prompt = "Write about the raw, unfiltered experience of being human".to_string();
            let meta_form = create_meta_form(None);  // No previous poem for first cycle
            
            let new_state = PoetState {
                current_cycle: 0,
                total_poems: 0,
                genesis_prompt,
                meta_form,
                last_updated: get_current_time(),
            };
            
            state.borrow_mut().insert(0, new_state.clone());
            new_state
        })
    } else {
        // Get existing state
        POET_STATE.with(|state| {
            state.borrow().get(&0)
                .ok_or("Poet state disappeared unexpectedly".to_string())
        })?
    };
    
    // Get previous poem if it exists (for reflection)
    let previous_poem = if poet_state.current_cycle > 0 {
        POEM_CYCLES.with(|cycles| {
            cycles.borrow()
                .get(&poet_state.current_cycle)
                .map(|cycle| cycle.poem.clone())
        })
    } else {
        None
    };
    
    // Determine current prompt
    let current_prompt = if poet_state.current_cycle == 0 {
        poet_state.genesis_prompt.clone()
    } else {
        POEM_CYCLES.with(|cycles| {
            cycles.borrow()
                .get(&poet_state.current_cycle)
                .map(|cycle| cycle.next_prompt.clone())
                .unwrap_or_else(|| "Write about lost prompts".to_string())
        })
    };
    
    // Create meta form with reflection on previous poem
    let meta_form = create_meta_form(previous_poem);
    
    // Apply meta form to create the full prompt
    let full_prompt = apply_meta_form(&meta_form, poet_state.current_cycle + 1, &current_prompt);
    
    // STEP 1: Get LLM response
    let messages = vec![ChatMessage::System {
        content: full_prompt
    }];
    
    let llm_response = ic_llm::chat(Model::Llama3_1_8B)
        .with_messages(messages)
        .send()
        .await;
    
    let raw_response = llm_response.message.content.unwrap_or_default();
    
    // STEP 2: Parse with multiple strategies
    let (poem, title, next_prompt, method) = {
        // First check if old markers are used - force immediate correction
        if has_old_markers(&raw_response) {
            // Force format correction
            let format_correction_prompt = create_format_correction_prompt(&raw_response);
            let correction_messages = vec![ChatMessage::System {
                content: format_correction_prompt
            }];
            
            let correction_result = ic_llm::chat(Model::Llama3_1_8B)
                .with_messages(correction_messages)
                .send()
                .await;
            
            if let Some(correction_response) = correction_result.message.content {
                // Try parsing the corrected response
                if let Ok((p, t, n)) = parse_with_labels(&correction_response) {
                    (p, t, n, GenerationMethod::Corrected)
                } else if let Ok((p, t, n)) = parse_with_heuristics(&correction_response) {
                    (p, t, n, GenerationMethod::Corrected)
                } else {
                    // Still failed - use algorithmic fallback
                    let (p, t, n) = generate_algorithmic_fallback(&raw_response, poet_state.current_cycle + 1, &current_prompt);
                    (p, t, n, GenerationMethod::Algorithmic)
                }
            } else {
                // Correction failed - use algorithmic fallback
                let (p, t, n) = generate_algorithmic_fallback(&raw_response, poet_state.current_cycle + 1, &current_prompt);
                (p, t, n, GenerationMethod::Algorithmic)
            }
        }
        // Try primary parsing with database labels
        else if let Ok((p, t, n)) = parse_with_labels(&raw_response) {
            (p, t, n, GenerationMethod::Primary)
        }
        // Try heuristic parsing
        else if let Ok((p, t, n)) = parse_with_heuristics(&raw_response) {
            (p, t, n, GenerationMethod::Fallback)
        }
        // Try general correction
        else {
            let correction_prompt = create_correction_prompt(&raw_response);
            let correction_messages = vec![ChatMessage::System {
                content: correction_prompt
            }];
            
            let correction_result = ic_llm::chat(Model::Llama3_1_8B)
                .with_messages(correction_messages)
                .send()
                .await;
            
            if let Some(correction_response) = correction_result.message.content {
                if let Ok((p, t, n)) = parse_with_labels(&correction_response) {
                    (p, t, n, GenerationMethod::Corrected)
                } else if let Ok((p, t, n)) = parse_with_heuristics(&correction_response) {
                    (p, t, n, GenerationMethod::Corrected)
                } else {
                    // Ultimate fallback - algorithmic generation
                    let (p, t, n) = generate_algorithmic_fallback(&raw_response, poet_state.current_cycle + 1, &current_prompt);
                    (p, t, n, GenerationMethod::Algorithmic)
                }
            } else {
                // If correction fails, use algorithmic generation
                let (p, t, n) = generate_algorithmic_fallback(&raw_response, poet_state.current_cycle + 1, &current_prompt);
                (p, t, n, GenerationMethod::Algorithmic)
            }
        }
    };
    
    // STEP 3: Create and store the poem cycle (GUARANTEED to have valid data)
    let new_cycle_id = poet_state.current_cycle + 1;
    let poem_cycle = PoemCycle {
        id: new_cycle_id,
        cycle_number: new_cycle_id,
        poem: poem.trim().to_string(),
        title: title.trim().to_string(),
        next_prompt: next_prompt.trim().to_string(),
        created_at: get_current_time(),
        raw_response: raw_response.chars().take(5000).collect(), // Store first 5000 chars for debugging
        generation_method: method,
    };
    
    // Store the poem cycle
    POEM_CYCLES.with(|cycles| {
        cycles.borrow_mut().insert(new_cycle_id, poem_cycle.clone());
    });
    
    // Update poet state
    let updated_state = PoetState {
        current_cycle: new_cycle_id,
        total_poems: poet_state.total_poems + 1,
        genesis_prompt: poet_state.genesis_prompt,
        meta_form: poet_state.meta_form,
        last_updated: get_current_time(),
    };
    
    POET_STATE.with(|state| {
        state.borrow_mut().insert(0, updated_state);
    });
    
    Ok(poem_cycle)
}

// Initialize the poet
#[init]
fn init() {
    let genesis_prompt = "Write about the raw, unfiltered experience of being human".to_string();
    let meta_form = create_meta_form(None);  // No previous poem for initialization
    
    let poet_state = PoetState {
        current_cycle: 0,
        total_poems: 0,
        genesis_prompt,
        meta_form,
        last_updated: get_current_time(),
    };
    
    POET_STATE.with(|state| {
        state.borrow_mut().insert(0, poet_state);
    });
}

// Query methods
#[query]
fn get_current_poem() -> Option<PoemCycle> {
    let poet_state = POET_STATE.with(|state| state.borrow().get(&0))?;
    
    if poet_state.current_cycle == 0 {
        return None;
    }
    
    POEM_CYCLES.with(|cycles| {
        cycles.borrow().get(&poet_state.current_cycle)
    })
}

#[query]
fn get_all_poems() -> Vec<PoemCycle> {
    POEM_CYCLES.with(|cycles| {
        cycles.borrow()
            .iter()
            .map(|(_, cycle)| cycle)
            .collect()
    })
}

#[query]
fn get_poet_state() -> Option<PoetState> {
    POET_STATE.with(|state| state.borrow().get(&0))
}

#[query]
fn get_poem_by_cycle(cycle_number: u64) -> Option<PoemCycle> {
    POEM_CYCLES.with(|cycles| {
        cycles.borrow().get(&cycle_number)
    })
}

#[query]
fn get_poem_count() -> u64 {
    POET_STATE.with(|state| {
        state.borrow().get(&0).map(|s| s.total_poems).unwrap_or(0)
    })
}

// Analytics query to see how well parsing is working
#[derive(CandidType, Deserialize, Serialize)]
pub struct GenerationStats {
    pub total_poems: u64,
    pub primary_success: u64,
    pub fallback_used: u64,
    pub correction_used: u64,
    pub algorithmic_used: u64,
}

#[query]
fn get_generation_stats() -> GenerationStats {
    let poems = get_all_poems();
    let mut stats = GenerationStats {
        total_poems: poems.len() as u64,
        primary_success: 0,
        fallback_used: 0,
        correction_used: 0,
        algorithmic_used: 0,
    };
    
    for poem in poems {
        match poem.generation_method {
            GenerationMethod::Primary => stats.primary_success += 1,
            GenerationMethod::Fallback => stats.fallback_used += 1,
            GenerationMethod::Corrected => stats.correction_used += 1,
            GenerationMethod::Algorithmic => stats.algorithmic_used += 1,
        }
    }
    
    stats
}

// Update methods
#[update]
fn reset_poet() -> bool {
    // Clear all poems
    POEM_CYCLES.with(|cycles| {
        let memory = MEMORY_MANAGER.with(|m| m.borrow().get(POEM_CYCLES_MEMORY_ID));
        let new_map = StableBTreeMap::init(memory);
        *cycles.borrow_mut() = new_map;
    });
    
    // Reset state with fresh meta form
    let genesis_prompt = "Write about the raw, unfiltered experience of being human".to_string();
    let meta_form = create_meta_form(None);  // No previous poem after reset
    
    let poet_state = PoetState {
        current_cycle: 0,
        total_poems: 0,
        genesis_prompt,
        meta_form,
        last_updated: get_current_time(),
    };
    
    POET_STATE.with(|state| {
        state.borrow_mut().insert(0, poet_state);
    });
    
    true
}

// Manual override for testing - set specific next prompt
#[update]
fn set_next_prompt(next_prompt: String) -> bool {
    POET_STATE.with(|state| {
        if let Some(poet_state) = state.borrow().get(&0) {
            if poet_state.current_cycle > 0 {
                POEM_CYCLES.with(|cycles| {
                    if let Some(mut current_cycle) = cycles.borrow().get(&poet_state.current_cycle) {
                        current_cycle.next_prompt = next_prompt;
                        cycles.borrow_mut().insert(poet_state.current_cycle, current_cycle);
                        true
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        } else {
            false
        }
    })
}

// Get raw response for debugging
#[query]
fn get_raw_response(cycle_number: u64) -> Option<String> {
    POEM_CYCLES.with(|cycles| {
        cycles.borrow()
            .get(&cycle_number)
            .map(|cycle| cycle.raw_response.clone())
    })
}

#[pre_upgrade]
fn pre_upgrade() {
    // State is automatically preserved in stable memory
}

#[post_upgrade]
fn post_upgrade() {
    // State is automatically restored from stable memory
}

// Export the Candid interface
ic_cdk::export_candid!();