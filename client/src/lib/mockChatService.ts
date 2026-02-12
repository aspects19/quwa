// Mock chat service for demo purposes
// This simulates the backend AI responses with realistic rare disease analysis
// Based on the internal pipeline structure for comprehensive demo

interface MockResponse {
  thinkingSteps: string[];
  response: string;
}

type StreamEvent = 
  | { type: 'thinking'; data: { step: string } }
  | { type: 'response'; data: { content: string } }
  | { type: 'done'; data: Record<string, never> };

// Comprehensive hEDS case study based on real clinical pipeline
const comprehensiveHEDSCase: MockResponse = {
  thinkingSteps: [
    "Parsing patient input: Female, 26 years old. Extracting chief complaints: chronic joint pain since adolescence, recurrent shoulder and knee subluxations, generalized joint hypermobility, chronic fatigue, orthostatic dizziness, frequent sprains with minimal trauma",
    "Processing physical examination findings: Beighton Score 7/9 (positive for hypermobility), skin characteristics (soft texture, mild hyperextensibility, no atrophic scarring), photographic evidence of passive thumb-to-forearm apposition confirmed",
    "Analyzing diagnostic results: MRI negative for inflammatory arthritis, laboratory inflammatory markers within normal limits. Critical observation: Normal imaging despite severe symptoms suggests non-inflammatory connective tissue disorder",
    "Normalizing clinical features into standardized ontology: generalized_joint_hypermobility, chronic_noninflammatory_pain, joint_instability, recurrent_subluxations, autonomic_dysfunction, positive_family_history, soft_skin, normal_inflammatory_markers",
    "Initiating knowledge source queries across multiple databases: Orphanet (rare disease phenotypes), GeneReviews (genetic disorder monographs), Curated EDS subtype clinical dataset (diagnostic criteria)",
    "Orphanet semantic search returned 847 potential connective tissue disorder matches. Filtering by symptom overlap and phenotypic similarity... Narrowed to 23 high-confidence candidates",
    "GeneReviews cross-reference complete: 23 relevant genetic profiles retrieved. Extracting diagnostic criteria and inheritance patterns for differential analysis",
    "Semantic similarity calculation complete. Top 5 candidates ranked by phenotype match: (1) Hypermobile EDS [0.94], (2) Hypermobility Spectrum Disorder [0.78], (3) Classical EDS [0.61], (4) Marfan Syndrome [0.52], (5) Fibromyalgia [0.47]",
    "Pattern matching analysis for primary candidate (hEDS): Beighton 7/9 STRONGLY SUPPORTS (criterion met), recurrent subluxations STRONGLY SUPPORTS (major criterion), chronic pain disproportionate to imaging SUPPORTS, autonomic dysfunction SUPPORTS (70-80% prevalence in hEDS), family history SUPPORTS (autosomal dominant), soft hyperextensible skin without scarring SUPPORTS, normal inflammatory markers SUPPORTS",
    "Evaluating alternative: Hypermobility Spectrum Disorder (HSD). Assessment: Patient severity (Beighton 7/9, multiple subluxations, autonomic involvement) EXCEEDS HSD threshold. HSD is exclusion diagnosis when hEDS criteria not met. This patient meets hEDS criteria. Disqualification: Disease severity too high. \nSTATUS: UNLIKELY",
    "Evaluating alternative: Classical Ehlers-Danlos Syndrome (cEDS). Assessment: Patient shows mild skin hyperextensibility but LACKS atrophic scarring (hallmark of cEDS caused by COL5A1/COL5A2 mutations). Disqualification: Absence of characteristic widened atrophic scars.\n STATUS: UNLIKELY",
    "Evaluating alternative: Marfan Syndrome. Assessment: No marfanoid habitus (tall stature, arachnodactyly, pectus abnormality), no ectopia lentis (lens dislocation), no cardiovascular abnormalities (aortic root dilation). Patient does not meet Ghent nosology criteria. Disqualification: Absence of skeletal, ocular, and cardiovascular features.\n STATUS: UNLIKELY",
    "Evaluating alternative: Fibromyalgia. Assessment: Patient has chronic widespread pain BUT also objective hypermobility (Beighton 7/9) and recurrent subluxations. Fibromyalgia cannot explain: measurable joint hypermobility, structural subluxations, positive family history. Disqualification: Unable to account for objective structural findings. May co-occur but not primary diagnosis.\n STATUS: UNLIKELY",
    "Cross-referencing absent clinical features: No marfanoid habitus (rules out Marfan), no ectopia lentis (rules out Marfan/homocystinuria), no atrophic scarring (rules out cEDS, supports hEDS), no vascular fragility (rules out vascular EDS), no elevated inflammatory markers (rules out inflammatory arthritis)",
    "Clinical context evaluation: hEDS is frequently misdiagnosed as fibromyalgia, psychosomatic disorder, anxiety disorder, or chronic fatigue syndrome. Average diagnostic delay: 10-12 years from symptom onset. Critical note: NO genetic test exists for hEDS - diagnosis is entirely clinical per 2017 International Classification",
    "Diagnostic criteria verification against 2017 hEDS criteria: Criterion 1 (Generalized joint hypermobility - Beighton ≥5) ✓ MET, Criterion 2 (Systemic manifestations of connective tissue disorder) ✓ MET, Criterion 3 (Family history positive) ✓ MET. All three criteria satisfied",
    "Confidence assessment: HIGH confidence (0.94 similarity score) for Hypermobile Ehlers-Danlos Syndrome based on: positive diagnostic criteria verification, strong pattern match across multiple evidence points, systematic exclusion of high-probability alternatives",
    "Synthesizing clinical decision support recommendations: confirmatory evaluation pathway, supporting investigations for comorbidities, management and monitoring strategy"
  ],
  response: `**CASE ID:** demo-eds-001

## Primary Diagnosis

**Hypermobile Ehlers–Danlos Syndrome (hEDS)**  
*Confidence Level: HIGH*

Clinical presentation strongly consistent with hEDS based on:
- High Beighton score (7/9)
- Recurrent joint subluxations with minimal trauma
- Chronic pain disproportionate to imaging findings
- Autonomic dysfunction (orthostatic dizziness)
- Positive family history

---

## Recommended Actions

### Confirmatory Evaluation
1. **Clinical genetics or rheumatology referral** for formal assessment using 2017 hEDS diagnostic criteria
2. **Formal Beighton scoring** by trained connective tissue disorder specialist
3. **Family history documentation** and pedigree analysis

### Supporting Investigations
4. **Autonomic testing** - Screen for POTS or other dysautonomia
5. **Cardiac echocardiography** - Assess for mitral valve prolapse (common in hEDS)
6. **Physical therapy evaluation** - Joint stabilization and proprioceptive training program
7. **Baseline imaging** - Document current joint status

### Management Priorities
- Joint protection strategies and activity modification
- Pain management (preferably non-pharmacological)
- Monitor for associated conditions: MCAS, dysautonomia, GI dysmotility
- Patient education on avoiding high-impact activities

---

**Note:** This is clinical decision support only. Final diagnosis must be made by qualified healthcare professionals using established clinical criteria. There is currently no genetic test for hEDS - diagnosis is entirely clinical.`
};

// Fallback shorter response for testing
const quickTestResponse: MockResponse = {
  thinkingSteps: [
    "Analyzing patient presentation...",
    "Querying rare disease databases...",
    "Generating recommendations..."
  ],
  response: `**Quick Analysis**

Based on the symptoms described, I recommend comprehensive evaluation for connective tissue disorders, particularly Ehlers-Danlos Syndrome variants.

**Next Steps:**
- Clinical genetics referral
- Beighton score assessment
- Family history review

*This is clinical decision support only. Consult qualified healthcare professionals.*`
};

export function getMockResponse(message: string): MockResponse {
  // Use comprehensive case for detailed inputs
  if (message.length > 100) {
    return comprehensiveHEDSCase;
  }
  
  // Otherwise use quick response
  return quickTestResponse;
}

export async function* streamMockResponse(message: string): AsyncGenerator<StreamEvent> {
  const mockData = getMockResponse(message);
  
  // Stream thinking steps word-by-word for dynamic effect
  for (const step of mockData.thinkingSteps) {
    const words = step.split(' ');
    let accumulatedStep = '';
    
    for (let i = 0; i < words.length; i++) {
      accumulatedStep += words[i] + (i < words.length - 1 ? ' ' : '');
      yield { type: 'thinking', data: { step: accumulatedStep } };
      await sleep(50 + Math.random() * 30); // 50-80ms per word
    }
    
    // Pause between complete thinking steps
    await sleep(2000 + Math.random() * 10000); // 2-3s pause between steps
  }
  
  // Small pause before response
  await sleep(1000);
  
  // Stream response content word by word
  const words = mockData.response.split(' ');
  for (let i = 0; i < words.length; i++) {
    const content = words[i] + (i < words.length - 1 ? ' ' : '');
    yield { type: 'response', data: { content } };
    await sleep(25 + Math.random() * 15); // 25-40ms per word for smooth streaming
  }
  
  yield { type: 'done', data: {} };
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// Export the comprehensive case prompt for easy testing
export const DEMO_PROMPT = `Female patient, 26 years old, presenting with:

**Chief Complaints:**
- Chronic joint pain since adolescence
- Recurrent shoulder and knee subluxations
- Generalized joint hypermobility
- Chronic fatigue
- Orthostatic dizziness
- Frequent sprains with minimal trauma

**Physical Findings:**
- Beighton Score: 7/9
- Soft skin texture with mild hyperextensibility
- No atrophic scarring
- Passive thumb can touch forearm demonstrating hypermobility

**Diagnostic Results:**
- MRI (knees and shoulders): Unremarkable, no inflammatory arthritis
- Laboratory: Inflammatory markers within normal limits

**Family History:** Positive for similar symptoms

Please provide rare disease differential diagnosis and recommendations.`;

