use anyhow::{Result, Context};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct OrphanetDisorder {
    pub orpha_code: String,
    pub name: String,
    pub hpo_associations: Vec<HPOAssociation>,
}

#[derive(Debug, Clone)]
pub struct HPOAssociation {
    pub hpo_id: String,
    pub hpo_term: String,
    pub frequency: String,
}

impl OrphanetDisorder {
    /// Convert disorder to embedable text format
    pub fn to_embedable_text(&self) -> String {
        let mut text = format!(
            "Disease: {} (Orpha: {})\n\nClinical Signs and Symptoms:\n",
            self.name, self.orpha_code
        );
        
        for assoc in &self.hpo_associations {
            text.push_str(&format!(
                "- {} ({}) [{}]\n",
                assoc.hpo_term, assoc.frequency, assoc.hpo_id
            ));
        }
        
        text
    }
}

pub struct OrphanetProcessor {
    limit: Option<usize>,
}

impl OrphanetProcessor {
    pub fn new(limit: Option<usize>) -> Self {
        Self { limit }
    }
    
    /// Parse the Orphanet XML file and extract disorders
    pub fn parse_xml<P: AsRef<Path>>(&self, path: P) -> Result<Vec<OrphanetDisorder>> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read Orphanet XML file")?;
        
        let mut reader = Reader::from_str(&content);
        
        let mut disorders = Vec::new();
        let mut current_disorder: Option<OrphanetDisorder> = None;
        let mut current_hpo: Option<HPOAssociation> = None;
        
        let mut current_element = String::new();
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    current_element = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    
                    match current_element.as_str() {
                        "Disorder" => {
                            current_disorder = Some(OrphanetDisorder {
                                orpha_code: String::new(),
                                name: String::new(),
                                hpo_associations: Vec::new(),
                            });
                        }
                        "HPODisorderAssociation" => {
                            current_hpo = Some(HPOAssociation {
                                hpo_id: String::new(),
                                hpo_term: String::new(),
                                frequency: String::new(),
                            });
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap().to_string();
                    
                    match current_element.as_str() {
                        "OrphaCode" => {
                            if let Some(ref mut disorder) = current_disorder {
                                disorder.orpha_code = text;
                            }
                        }
                        "Name" => {
                            // Only capture disorder name (not other Name elements)
                            if let Some(ref mut disorder) = current_disorder {
                                if disorder.name.is_empty() {
                                    disorder.name = text;
                                }
                            }
                        }
                        "HPOId" => {
                            if let Some(ref mut hpo) = current_hpo {
                                hpo.hpo_id = text;
                            }
                        }
                        "HPOTerm" => {
                            if let Some(ref mut hpo) = current_hpo {
                                hpo.hpo_term = text;
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(e)) => {
                    let element = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    
                    match element.as_str() {
                        "HPODisorderAssociation" => {
                            if let (Some(disorder), Some(hpo)) = (&mut current_disorder, current_hpo.take()) {
                                disorder.hpo_associations.push(hpo);
                            }
                        }
                        "HPOFrequency" => {
                            // Capture the frequency from the Name element that follows
                            if let Some(ref mut hpo) = current_hpo {
                                // Frequency will be captured in the Name text event
                                if hpo.frequency.is_empty() {
                                    // Read ahead for the Name element
                                    let mut freq_buf = Vec::new();
                                    if let Ok(Event::Start(_)) = reader.read_event_into(&mut freq_buf) {
                                        if let Ok(Event::Text(freq_text)) = reader.read_event_into(&mut freq_buf) {
                                            hpo.frequency = freq_text.unescape().unwrap().to_string();
                                        }
                                    }
                                }
                            }
                        }
                        "Disorder" => {
                            if let Some(disorder) = current_disorder.take() {
                                // Only add disorders with HPO associations
                                if !disorder.hpo_associations.is_empty() {
                                    disorders.push(disorder);
                                    
                                    // Check limit
                                    if let Some(limit) = self.limit {
                                        if disorders.len() >= limit {
                                            tracing::info!("Reached limit of {} disorders", limit);
                                            return Ok(disorders);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    tracing::error!("XML parsing error at position {}: {:?}", reader.buffer_position(), e);
                    break;
                }
                _ => {}
            }
            buf.clear();
        }
        
        tracing::info!("Parsed {} disorders from Orphanet XML", disorders.len());
        Ok(disorders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disorder_to_text() {
        let disorder = OrphanetDisorder {
            orpha_code: "58".to_string(),
            name: "Alexander disease".to_string(),
            hpo_associations: vec![
                HPOAssociation {
                    hpo_id: "HP:0000256".to_string(),
                    hpo_term: "Macrocephaly".to_string(),
                    frequency: "Very frequent (99-80%)".to_string(),
                },
                HPOAssociation {
                    hpo_id: "HP:0001249".to_string(),
                    hpo_term: "Intellectual disability".to_string(),
                    frequency: "Very frequent (99-80%)".to_string(),
                },
            ],
        };
        
        let text = disorder.to_embedable_text();
        assert!(text.contains("Alexander disease"));
        assert!(text.contains("Orpha: 58"));
        assert!(text.contains("Macrocephaly"));
        assert!(text.contains("Very frequent"));
    }
}
