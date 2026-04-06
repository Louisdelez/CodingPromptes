# Guide Complet du Prompt Engineering : Principes, Techniques et Bonnes Pratiques (2025-2026)

---

## Table des matieres

1. [Fondamentaux du Prompt Engineering](#1-fondamentaux-du-prompt-engineering)
2. [Structures et Frameworks de Prompts](#2-structures-et-frameworks-de-prompts)
3. [Techniques Avancees](#3-techniques-avancees)
4. [Recommandations Officielles des Fournisseurs d'IA](#4-recommandations-officielles-des-fournisseurs-dia)
5. [Erreurs Courantes a Eviter](#5-erreurs-courantes-a-eviter)
6. [Sources](#6-sources)

---

## 1. Fondamentaux du Prompt Engineering

### 1.1 Qu'est-ce que le Prompt Engineering ?

Le prompt engineering est l'art et la science de formuler des instructions efficaces pour les modeles de langage (LLM) afin d'obtenir des reponses precises, pertinentes et exploitables. En 2025-2026, cette discipline a evolue bien au-dela de la simple formulation de questions : il s'agit desormais de concevoir des interactions structurees qui guident les modeles vers des resultats optimaux.

### 1.2 Les Principes Fondamentaux

#### Clarte et Precision

Le principe le plus fondamental est d'etre aussi precis que possible. Les meilleurs prompts minimisent les suppositions du modele en definissant clairement la tache, le contexte, le format souhaite et le ton. Comme le recommande Anthropic dans sa documentation officielle : **"Pensez a Claude comme un employe brillant mais nouveau qui manque de contexte sur vos normes et processus. Plus vous expliquez precisement ce que vous voulez, meilleur sera le resultat."**

La regle d'or d'Anthropic : montrez votre prompt a un collegue ayant peu de contexte sur la tache et demandez-lui de le suivre. S'il est confus, le modele le sera aussi.

#### Fournir du Contexte

Donner un contexte ou une motivation derriere vos instructions aide le modele a mieux comprendre vos objectifs. Par exemple, plutot que de dire simplement "N'utilisez JAMAIS de points de suspension", il est plus efficace d'expliquer : "Votre reponse sera lue par un moteur de synthese vocale, donc n'utilisez jamais de points de suspension car le moteur ne saura pas comment les prononcer." Le modele est suffisamment intelligent pour generaliser a partir de l'explication.

#### Structure et Organisation

Un bon prompt comporte plusieurs composantes cles :
- **Le role** : qui le modele doit incarner
- **Le ton** : le style de communication souhaite
- **La tache** : ce qui doit etre accompli
- **Le format** : la structure de la reponse attendue
- **Les contraintes** : les limites et restrictions
- **L'audience** : a qui la reponse s'adresse

#### Iteration et Raffinement

Le prompt engineering est un processus iteratif : on commence avec un prompt initial, on examine la reponse, puis on affine en ajustant la formulation, en ajoutant du contexte ou en simplifiant la demande. Il ne s'agit jamais de deviner le prompt parfait du premier coup, mais de raffiner progressivement par retour d'experience.

---

## 2. Structures et Frameworks de Prompts

Plusieurs frameworks structures ont ete developpes pour guider la creation de prompts efficaces. Chacun a ses forces et cas d'usage optimaux.

### 2.1 CO-STAR

**Composantes** : Context (Contexte), Objective (Objectif), Style, Tone (Ton), Audience, Response (Reponse)

**Cas d'usage** : Textes marketing, articles de blog, emails et tout contenu ou la voix et le ciblage de l'audience impactent directement la qualite.

**Forces** : Controle explicite du ton et de l'audience, empechant les decalages de communication. Les references de style permettent des patterns d'ecriture specifiques.

**Limites** : Pas de mecanisme de raisonnement multi-etapes ni de section d'exemples integree.

### 2.2 RISEN

**Composantes** : Role, Instructions, Steps (Etapes), End Goal (Objectif Final), Narrowing (Restriction)

**Cas d'usage** : Taches techniques multi-etapes, recherche, revues de code et flux de travail necessitant des operations sequentielles.

**Forces** : Force la definition explicite du processus, empechant les raccourcis de l'IA. L'element "End Goal" ancre les resultats a des objectifs mesurables. L'element "Steps" distingue ce framework en demandant au modele de montrer son travail en phases explicites avant de donner le resultat final.

**Limites** : Trop lourd pour les taches simples. Pas de specification de ton ou d'audience.

### 2.3 CRISPE

**Composantes** : Clarity (Clarte), Relevance (Pertinence), Iteration, Specificity (Specificite), Parameters (Parametres), Examples (Exemples)

Ce framework permet aux utilisateurs de creer des prompts efficaces qui equilibrent precision et creativite. En decomposant les taches en etapes logiques mais flexibles, CRISPE garantit des resultats actionnables et innovants.

### 2.4 APE

**Composantes** : Action, Purpose (Objectif), Expectation (Attente)

**Cas d'usage** : Iteration rapide, brainstorming, taches simples en un coup et flux de travail quotidiens.

**Forces** : Surcharge minimale. Le champ "Purpose" empeche de maniere unique les resultats sans but parmi les frameworks legers.

**Limites** : Pas de specification de role, d'audience ou de contexte.

### 2.5 RACE

**Composantes** : Role, Action, Context (Contexte), Expect (Attente)

**Cas d'usage** : Taches quotidiennes rapides ou la vitesse prime sur le peaufinage.

**Forces** : Le plus rapide a ecrire -- quatre champs, chacun d'une phrase. Le champ "Expect" force la definition de criteres de succes.

**Limites** : Pas de controle de style/ton ; manque de structure de processus multi-etapes.

### 2.6 CREATE

**Composantes** : Character (Personnage), Request (Requete), Examples (Exemples), Adjustments (Ajustements), Type, Extras

**Cas d'usage** : Travail creatif detaille necessitant un format, une voix ou un style correspondant a des materiaux de reference.

**Forces** : Composante d'exemples "few-shot" integree -- "montrer a l'IA a quoi ressemble un bon resultat est systematiquement la technique de prompting a plus fort impact."

**Limites** : Configuration la plus exigeante ; chevauchement des portees des composantes.

### 2.7 STOKE

**Composantes** : Situation, Task (Tache), Objective (Objectif), Knowledge (Connaissances), Examples (Exemples)

**Cas d'usage** : Taches d'expertise de domaine, redaction technique, analyse ou les connaissances specialisees determinent la precision.

**Forces** : La composante "Knowledge" comble l'ecart entre la definition de la tache et le contexte du domaine.

### 2.8 Guide de Selection

| Complexite | Framework recommande |
|---|---|
| Taches simples et rapides | APE (3 composantes) ou RACE (4 composantes) |
| Contenu de complexite moyenne | CO-STAR |
| Taches techniques multi-etapes | RISEN |
| Travail creatif avec exemples | CREATE ou STOKE |

**Specificites par modele** : GPT-4o excelle avec la structure pas-a-pas de RISEN. Claude integre naturellement la composante "Knowledge" de STOKE. Gemini performe mieux avec les contraintes explicites du "Narrowing" de RISEN ou des "Adjustments" de CREATE.

---

## 3. Techniques Avancees

### 3.1 Techniques de Base

#### Zero-Shot Prompting
Execution directe d'une tache sans fournir d'exemples. On demande au modele d'accomplir une tache en s'appuyant uniquement sur ses connaissances pre-entrainees.

*Exemple : "Analysez le sentiment : 'Le produit etait decevant mais la livraison rapide'"*

#### Few-Shot Prompting (Apprentissage par Exemples)
Fournir 2 a 5 exemples pour guider les patterns de reponse. Cela aide a etablir le format et le style de sortie desires. Anthropic recommande **3 a 5 exemples** pour de meilleurs resultats, en les enveloppant dans des balises `<example>` pour que le modele puisse les distinguer des instructions.

Les exemples doivent etre :
- **Pertinents** : refletant votre cas d'usage reel
- **Divers** : couvrant les cas limites et variant suffisamment pour eviter des patterns non intentionnels
- **Structures** : clairement delimites du reste du prompt

#### In-Context Learning (Apprentissage en Contexte)
Apprentissage de patterns directement a partir du contexte fourni, sans instructions explicites. Le modele deduit le pattern a partir des exemples et le generalise.

### 3.2 Techniques de Raisonnement

#### Chain-of-Thought (CoT) - Chaine de Pensee
Decompose les problemes complexes en etapes de raisonnement intermediaires. Particulierement efficace pour les problemes mathematiques et l'analyse logique. Le principe est de demander au modele de "reflechir etape par etape" avant de donner sa reponse finale.

*Exemple : "Reflechissez etape par etape : 1. Calculez la remise 2. Appliquez la taxe 3. Determinez le total"*

Anthropic recommande d'utiliser des balises structurees comme `<thinking>` et `<answer>` pour separer proprement le raisonnement du resultat final lorsque le mode "thinking" est desactive.

#### Self-Consistency (Auto-Coherence)
Ameliore le CoT en introduisant plusieurs chaines de raisonnement independantes a partir du meme prompt initial. L'idee est d'echantillonner plusieurs chemins de raisonnement divers et de selectionner la reponse la plus coherente. Cela booste les performances sur les taches d'arithmetique et de raisonnement de sens commun.

*Exemple : "Resolvez en utilisant 3 approches differentes, puis rapportez la reponse consensus"*

#### Tree of Thoughts (ToT) - Arbre de Pensees
Generalise le chain-of-thought en encourageant l'exploration de multiples branches de raisonnement simultanement. Le framework maintient un arbre de pensees ou chaque pensee represente une sequence linguistique coherente servant d'etape intermediaire vers la resolution. Le modele peut auto-evaluer sa progression et utiliser des algorithmes de recherche pour une exploration systematique avec retour en arriere.

**Cas d'usage** : Environnements necessitant une exploration large et un elagage systematique d'idees, resolution creative de problemes.

#### ReAct (Reasoning + Acting - Raisonnement + Action)
Combine le raisonnement analytique avec des capacites d'action via des outils externes. A chaque iteration :
1. Le modele raisonne sur les observations precedentes
2. Il determine la prochaine action
3. L'action est executee par des systemes externes
4. Les observations resultantes sont reintroduites dans la boucle

L'approche la plus performante combine ReAct avec le Chain-of-Thought, permettant d'utiliser a la fois les connaissances internes et les informations externes obtenues pendant le raisonnement.

### 3.3 Techniques Avancees Complementaires

#### Meta Prompting
Utiliser l'IA pour optimiser et ameliorer ses propres prompts. On demande au modele de critiquer et d'ameliorer un prompt donne en termes de clarte, d'exemples et d'efficacite du format.

#### Prompt Chaining (Chainement de Prompts)
Prompts sequentiels connectes decomposant des flux de travail complexes en plusieurs etapes. Avec les capacites de reflexion adaptative et d'orchestration de sous-agents des modeles modernes, la plupart des raisonnements multi-etapes sont geres en interne. Le chainement explicite reste utile quand on a besoin d'inspecter des resultats intermediaires ou d'imposer une structure de pipeline specifique.

Le pattern le plus courant est l'**auto-correction** : generer un brouillon, faire reviser par le modele selon des criteres, puis raffiner sur la base de la revue.

#### Retrieval Augmented Generation (RAG)
Combine des sources de connaissances externes avec les capacites du modele pour des reponses fondees. Particulierement utile pour repondre a des questions en s'appuyant sur des documents d'entreprise avec citations de sources.

#### Automatic Reasoning and Tool-use (ART)
Selection et deploiement intelligents d'outils appropries pour des exigences de taches specifiques, automatisant le choix entre calculatrice, recherche web, ou autres outils selon le besoin.

#### Generate Knowledge Prompting
Faire produire au modele des informations de base pertinentes avant de repondre a la question principale, enrichissant ainsi le contexte de raisonnement.

#### Prompting Multimodal
Combiner texte, images, audio et video dans un meme prompt -- une tendance majeure de 2025-2026 permettant une assistance IA plus complete et nuancee.

---

## 4. Recommandations Officielles des Fournisseurs d'IA

### 4.1 Anthropic (Claude)

La documentation officielle d'Anthropic pour Claude Opus 4.6, Sonnet 4.6 et Haiku 4.5 constitue la reference la plus complete. Voici les principes cles :

#### Principes Generaux
- **Etre clair et direct** : Claude repond bien aux instructions claires et explicites. Si vous voulez un comportement "au-dela des attentes", demandez-le explicitement.
- **Ajouter du contexte** : Expliquer le "pourquoi" des instructions aide Claude a mieux cibler ses reponses.
- **Utiliser des exemples efficacement** : 3 a 5 exemples bien construits ameliorent considerablement la precision et la coherence.
- **Structurer avec des balises XML** : Les balises XML aident Claude a analyser les prompts complexes sans ambiguite (`<instructions>`, `<context>`, `<input>`).
- **Donner un role** : Definir un role dans le prompt systeme concentre le comportement et le ton de Claude.

#### Gestion du Contexte Long
- Placer les donnees longues **en haut** du prompt, au-dessus des instructions et exemples (amelioration jusqu'a 30%).
- Structurer les documents multiples avec des balises XML (`<document>`, `<document_content>`, `<source>`).
- Demander au modele de **citer les passages pertinents** avant d'effectuer sa tache pour mieux filtrer le bruit.

#### Controle du Format de Sortie
- Dire au modele **ce qu'il doit faire** plutot que ce qu'il ne doit pas faire.
- Utiliser des indicateurs de format XML.
- Adapter le style du prompt au style de sortie desire : le formatage de votre prompt influence le style de reponse de Claude.

#### Reflexion Adaptative (Adaptive Thinking)
Les modeles Claude 4.6 utilisent une reflexion adaptative ou Claude decide dynamiquement quand et combien reflechir. Le parametre `effort` controle la profondeur de reflexion. Plutot que de prescrire des etapes detaillees, un prompt comme "reflechissez en profondeur" produit souvent un meilleur raisonnement.

#### Systemes Agentiques
- Claude excelle dans le **raisonnement sur de longues sequences** avec un suivi d'etat exceptionnel.
- Utiliser des formats structures (JSON) pour les donnees d'etat et du texte libre pour les notes de progression.
- Git est recommande pour le suivi d'etat entre sessions multiples.
- Guider l'equilibre entre **autonomie et securite** : demander confirmation pour les actions irreversibles.

### 4.2 OpenAI (GPT)

Les recommandations officielles d'OpenAI mettent l'accent sur :

#### Strategies Principales
- **Ecrire des instructions claires** : Etre tres specifique ; plus le prompt est descriptif et detaille, meilleurs sont les resultats.
- **Decomposer les taches complexes** : Pour les grandes taches avec de nombreuses sous-taches, les decomposer en sous-taches plus simples et construire progressivement.
- **Approche iterative** : Commencer avec un prompt initial, examiner la reponse, et affiner en ajustant la formulation, ajoutant du contexte, ou simplifiant.
- **Utiliser les modeles les plus recents** : Les modeles plus recents tendent a etre plus faciles a guider par prompt engineering.
- **Temperature** : Pour les cas d'usage factuels (extraction de donnees, Q&A), une temperature de 0 est optimale. Des temperatures plus elevees produisent des sorties plus creatives mais moins previsibles.

### 4.3 Google (Gemini)

Les pratiques officielles de Google pour Gemini 3 (annonce en novembre 2025) soulignent :

- **Etre direct** : Gemini 3 suit les instructions courtes et directes beaucoup mieux que Gemini 2.x. Beaucoup de prompts Gemini 2.x peuvent etre raccourcis significativement.
- **Structure coherente** : Utiliser des delimiteurs clairs (balises XML ou titres Markdown) pour separer les parties du prompt, en maintenant un format uniforme.
- **Framework PTCF** : Gemini performe mieux avec le framework Persona, Task (Tache), Context (Contexte), Format -- la methode la plus fiable pour ameliorer la qualite des sorties.
- **Reponses directes par defaut** : Gemini 3 fournit des reponses directes et efficaces par defaut ; si vous voulez une reponse plus conversationnelle ou detaillee, demandez-le explicitement.
- **Exemples few-shot** : Eviter d'inclure trop d'exemples car le modele peut sur-apprendre, et assurer une structure coherente entre tous les exemples.

---

## 5. Erreurs Courantes a Eviter

### 5.1 Prompts Trop Vagues

L'erreur la plus courante est de manquer de precision. Demander "un resume" sans specifier la longueur, le format ou le focus laisse trop de place a l'interpretation. Il faut traiter le prompt comme des instructions donnees a un assistant, en incluant le format, le nombre de mots, l'audience cible et le ton.

**Mauvais** : "Fais-moi un resume de ce texte"
**Bon** : "Resume ce texte en 3 paragraphes pour un public de managers non techniques, en mettant l'accent sur les implications financieres"

### 5.2 Contexte Insuffisant

Fournir trop peu d'informations pousse le modele a generer des reponses generiques ou hors cible. Beaucoup d'utilisateurs traitent le modele comme un moteur de recherche plutot que comme un partenaire conversationnel. Sans contexte adequat, le modele se rabat sur des reponses generiques.

### 5.3 Ne Pas Iterer

Traiter le prompt engineering comme un processus en un coup est une erreur majeure. Les prompts ne sont jamais parfaits du premier essai. Il faut utiliser les reponses initiales comme retour d'information et raffiner iterativement pour une meilleure precision.

### 5.4 Surcharge d'un Seul Prompt

Rendre le prompt trop complexe ou y inclure plusieurs taches non liees confond le modele et produit des reponses alambiquees ou non pertinentes. Mieux vaut decomposer en plusieurs prompts cibles.

### 5.5 Ignorer les Limites du Modele

Supposer que l'IA "sait" toujours de quoi elle parle est une erreur critique. L'IA genere des reponses basees sur des patterns dans les donnees, pas sur une comprehension reelle ou des faits verifies. Il faut eviter les prompts necessitant des donnees en temps reel, des opinions subjectives ou des connaissances hautement specialisees hors de l'entrainement du modele.

### 5.6 Format de Sortie Non Specifie

Obtenir le bon contenu dans le mauvais format est un piege classique. Les LLM ne devinent pas votre structure de sortie preferee -- il faut la specifier explicitement (tableau, liste a puces, JSON, prose, etc.).

### 5.7 Instructions Negatives

Utiliser des instructions negatives ("ne faites pas X") au lieu de formulations positives ("faites Y a la place") est moins efficace. Anthropic le confirme : il vaut mieux dire "Votre reponse doit etre composee de paragraphes en prose fluide" plutot que "N'utilisez pas de markdown dans votre reponse."

### 5.8 Negliger la Structure du Prompt

Un prompt non structure produit des reponses desorganisees. Structurer logiquement avec des puces, des listes numerotees ou des instructions pas-a-pas ameliore significativement la qualite des resultats.

### 5.9 Negliger l'Audience et l'Objectif

Ne pas adapter le prompt a l'audience specifique ou au cas d'usage prevu reduit l'efficacite. La complexite et le style doivent correspondre au niveau de connaissance de l'utilisateur final et a l'objectif de la reponse.

### 5.10 Sur-Ingenierie (Specifique aux Systemes Agentiques)

Anthropic avertit que les modeles recents (Claude Opus 4.5 et 4.6) ont tendance a sur-ingenieriser en creant des fichiers supplementaires, ajoutant des abstractions inutiles ou integrant une flexibilite non demandee. Si des prompts precedents encourageaient le modele a etre plus exhaustif, il faut moderer ces instructions avec les modeles actuels qui sont deja significativement plus proactifs.

---

## 6. Sources

- [Anthropic - Prompting Best Practices (Documentation Officielle Claude 4)](https://platform.claude.com/docs/en/docs/build-with-claude/prompt-engineering/claude-4-best-practices)
- [Anthropic - Prompt Engineering Overview](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/overview)
- [Anthropic - Interactive Prompt Engineering Tutorial (GitHub)](https://github.com/anthropics/prompt-eng-interactive-tutorial)
- [OpenAI - Best Practices for Prompt Engineering](https://help.openai.com/en/articles/6654000-best-practices-for-prompt-engineering-with-the-openai-api)
- [OpenAI - Prompt Engineering Guide](https://platform.openai.com/docs/guides/prompt-engineering)
- [Google - Prompt Design Strategies (Gemini API)](https://ai.google.dev/gemini-api/docs/prompting-strategies)
- [Google - Gemini 3 Prompting Guide](https://docs.cloud.google.com/vertex-ai/generative-ai/docs/start/gemini-3-prompting-guide)
- [Prompt Engineering Guide (promptingguide.ai)](https://www.promptingguide.ai/)
- [Lakera - The Ultimate Guide to Prompt Engineering 2026](https://www.lakera.ai/blog/prompt-engineering-guide)
- [Promplify - Prompt Engineering Frameworks Compared](https://promplify.ai/blog/prompt-engineering-frameworks-compared/)
- [DataUnboxed - 15 Essential Prompt Engineering Techniques](https://www.dataunboxed.io/blog/the-complete-guide-to-prompt-engineering-15-essential-techniques-for-2025)
- [Penlify - CRISPE and Other Prompt Frameworks](https://penlify.com/pen/crispe-and-other-prompt-frameworks-which-actually-work-with-claude-and-gpt-4o/)
- [CodeSignal - Prompt Engineering Best Practices 2025](https://codesignal.com/blog/prompt-engineering-best-practices-2025/)
- [MxMoritz - Common Mistakes in Prompt Engineering](https://www.mxmoritz.com/article/common-mistakes-in-prompt-engineering)
- [Calmops - Prompt Engineering Patterns: CoT, ReAct, ToT](https://calmops.com/ai/prompt-engineering-patterns-cot-react-tot/)
- [Philipp Schmid - Gemini 3 Prompting Best Practices](https://www.philschmid.de/gemini-3-prompt-practices)
