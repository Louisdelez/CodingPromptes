# Prompt IDE — Guide d'utilisation

> Votre atelier de creation de prompts optimises pour l'IA

---

## Table des matieres

| # | Section | Ligne |
|---|---------|-------|
| # | Section | Ligne |
|---|---------|-------|
| 1 | [Demarrage rapide](#1-demarrage-rapide) | L.33 |
| 2 | [Decouverte de l'interface](#2-decouverte-de-linterface) | L.63 |
| 3 | [Creer votre premier prompt](#3-creer-votre-premier-prompt) | L.97 |
| 4 | [Les blocs : coeur de l'editeur](#4-les-blocs--coeur-de-lediteur) | L.137 |
| 5 | [Le systeme de variables](#5-le-systeme-de-variables) | L.172 |
| 6 | [Utiliser et creer des frameworks](#6-utiliser-et-creer-des-frameworks) | L.204 |
| 7 | [Le preview en temps reel](#7-le-preview-en-temps-reel) | L.275 |
| 8 | [Tester votre prompt (Playground)](#8-tester-votre-prompt-playground) | L.294 |
| 9 | [Comparer plusieurs modeles](#9-comparer-plusieurs-modeles) | L.367 |
| 10 | [Optimiser votre prompt avec l'IA](#10-optimiser-votre-prompt-avec-lia) | L.397 |
| 11 | [Valider votre prompt (Linting)](#11-valider-votre-prompt-linting) | L.428 |
| 12 | [Organiser vos prompts par projet](#12-organiser-vos-prompts-par-projet) | L.456 |
| 13 | [Versionner votre travail](#13-versionner-votre-travail) | L.557 |
| 14 | [Exporter votre prompt](#14-exporter-votre-prompt) | L.584 |
| 15 | [Dicter vos prompts (Speech-to-Text)](#15-dicter-vos-prompts-speech-to-text) | L.628 |
| 16 | [Installer le serveur local (Prompt AI Server)](#16-installer-le-serveur-local-prompt-ai-server) | L.684 |
| 17 | [Le compteur de tokens](#17-le-compteur-de-tokens) | L.789 |
| 18 | [Raccourcis et astuces](#18-raccourcis-et-astuces) | L.822 |
| 19 | [Import un prompt](#19-import-un-prompt) | |
| 20 | [Reponses en streaming](#20-reponses-en-streaming) | |
| 21 | [Historique des executions](#21-historique-des-executions) | |
| 22 | [Chainer des prompts (Workflows)](#22-chainer-des-prompts-workflows) | |
| 23 | [Mode Conversation (Chat)](#23-mode-conversation-chat) | |
| 24 | [Statistiques](#24-statistiques) | |
| 25 | [Application mobile (Responsive)](#25-application-mobile-responsive) | |
| 26 | [Installation hors-ligne (PWA)](#26-installation-hors-ligne-pwa) | |
| 27 | [Deployer avec Docker](#27-deployer-avec-docker) | |
| 28 | [FAQ et depannage](#28-faq-et-depannage) | |

---

## 1. Demarrage rapide

### Lancer l'application

```bash
cd prompt-ide
npm install    # Premiere fois uniquement
npm run dev    # Lance le serveur
```

Ouvrez **http://localhost:5173/** dans votre navigateur.

### En 30 secondes

1. L'application demarre avec 3 blocs vides : **Role**, **Contexte**, **Tache**
2. Ecrivez dans chaque bloc
3. Regardez le prompt compile dans le panneau **Preview** a droite
4. Cliquez sur **Copier** pour utiliser votre prompt

### En 2 minutes (workflow complet)

1. Cliquez l'icone **dossier+** dans la barre gauche pour creer un **projet**
2. Cliquez **+** sur le projet pour creer un premier prompt dedans
3. Editez vos blocs, testez dans le Playground
4. Creez d'autres prompts dans le meme projet pour votre workflow complet

C'est tout ! Le reste de ce guide couvre les fonctionnalites avancees.

---

## 2. Decouverte de l'interface

L'interface est organisee en 3 zones :

```
┌──────────┬────────────────────────────┬──────────────┐
│          │                            │              │
│  GAUCHE  │         CENTRE             │    DROITE    │
│          │                            │              │
│ 📂 Projet A   Vos blocs de prompt    │  Preview     │
│   📄 Prompt 1 (Role, Contexte, etc.) │  Playground  │
│   📄 Prompt 2                        │  IA          │
│ 📂 Projet B   Zone d'edition         │  Lint        │
│   📄 ...      principale             │  Export      │
│ Frame-  │                            │              │
│ works   │   Variables detectees      │              │
│ Versions├────────────────────────────┤              │
│         │ Tokens: 247  ~$0.002  GPT  │              │
└─────────┴────────────────────────────┴──────────────┘
```

### La barre d'en-tete

- **Logo Prompt IDE** : identite de l'application
- **Nom du projet** : cliquez dessus pour le renommer
- **Indicateur de sauvegarde** : "Sauvegarde..." apparait brievement lors de l'enregistrement automatique
- **Boutons panneaux** : masquer/afficher les panneaux gauche et droit pour plus d'espace

### Masquer les panneaux lateraux

Cliquez sur les icones `[<` et `>]` dans la barre d'en-tete pour masquer un panneau lateral. Utile sur petit ecran ou pour se concentrer sur l'edition.

---

## 3. Creer votre premier prompt

### Etape 1 : Definir le role

Dans le premier bloc **Role / Persona**, decrivez qui l'IA doit etre :

```
Tu es un redacteur web senior specialise en SEO et en copywriting 
persuasif, avec 10 ans d'experience dans le e-commerce.
```

### Etape 2 : Donner du contexte

Dans le bloc **Contexte**, fournissez les informations de fond :

```
L'entreprise {{nom_entreprise}} vend des {{produit}} en ligne.
Le site reçoit 50 000 visiteurs par mois mais le taux de 
conversion est de seulement 1.2%.
```

> Remarquez les `{{nom_entreprise}}` et `{{produit}}` — ce sont des **variables**. Elles seront detectees automatiquement (voir section 5).

### Etape 3 : Enoncer la tache

Dans le bloc **Tache / Directive**, soyez precis sur ce que vous attendez :

```
Redige 5 fiches produit optimisees SEO pour les pages les plus 
visitees. Chaque fiche doit inclure un titre H1 accrocheur, une 
meta-description de 155 caracteres, et 3 paragraphes de 
description persuasive.
```

### Etape 4 : Verifier et copier

Regardez le panneau **Preview** a droite : votre prompt est automatiquement assemble. Cliquez **Copier** pour l'utiliser dans ChatGPT, Claude, ou tout autre outil.

---

## 4. Les blocs : coeur de l'editeur

### Les 6 types de blocs

| Bloc | Couleur | Quand l'utiliser |
|------|---------|-----------------|
| **Role / Persona** | Violet | Definir qui l'IA doit etre, son expertise, son ton |
| **Contexte** | Bleu | Informations de fond, donnees, situation |
| **Tache / Directive** | Vert | L'action principale a accomplir |
| **Exemples (Few-shot)** | Ambre | Montrer des exemples d'entree/sortie attendus |
| **Contraintes** | Rouge | Limites, regles, restrictions, interdictions |
| **Format de sortie** | Gris | Structure attendue (JSON, liste, tableau, etc.) |

### Ajouter un bloc

Cliquez sur le bouton **+ Ajouter un bloc** en bas de la zone d'edition. Un menu s'ouvre avec les 6 types disponibles. Vous pouvez ajouter autant de blocs que vous voulez, y compris plusieurs blocs du meme type.

### Reorganiser les blocs

Chaque bloc a une poignee a gauche (icone `≡`). **Cliquez-glissez** cette poignee pour deplacer un bloc vers le haut ou le bas. L'ordre des blocs determine l'ordre dans le prompt compile.

### Activer / Desactiver un bloc

Survolez un bloc et cliquez sur l'icone **oeil** dans le coin superieur droit :
- **Oeil ouvert** = le bloc est inclus dans le prompt compile
- **Oeil ferme** = le bloc est garde en memoire mais exclus de la compilation

C'est utile pour tester l'impact d'un bloc sans le supprimer. Par exemple, desactivez le bloc Exemples pour voir comment le modele repond sans few-shot.

### Supprimer un bloc

Survolez un bloc et cliquez sur l'icone **corbeille** rouge. La suppression est immediate.

---

## 5. Le systeme de variables

### Creer une variable

Ecrivez `{{nom_de_variable}}` n'importe ou dans un bloc. La variable est automatiquement detectee.

**Exemples :**
```
Analyse le site {{url}} et compare avec {{concurrent}}
```

### Renseigner les valeurs

Sous les blocs, le panneau **Variables** apparait automatiquement des qu'une variable est detectee. Remplissez les champs :

```
{{url}}        → https://monsite.com
{{concurrent}} → amazon.fr
```

Les variables sont remplacees dans le prompt compile (visible dans le Preview).

### Auto-completion

Dans l'editeur de bloc, tapez `{{` et les variables existantes vous sont proposees en auto-completion. Selectionnez avec les fleches et Entree.

### Variables non resolues

Si une variable n'a pas de valeur, le compteur de tokens affiche un avertissement en orange. Le linting vous previent egalement.

---

## 6. Utiliser et creer des frameworks

### Qu'est-ce qu'un framework ?

Un framework est une structure pre-definie qui organise votre prompt selon une methode eprouvee. Au lieu de partir d'une page blanche, vous remplissez des sections guidees.

### Les 6 frameworks integres

Ouvrez le panneau gauche (onglet **Frameworks**), section **Frameworks integres** :

| Framework | Ideal pour | Sections |
|-----------|-----------|----------|
| **CO-STAR** | Marketing, emails, articles | Contexte, Objectif, Style, Ton, Audience, Reponse |
| **RISEN** | Taches techniques, multi-etapes | Role, Instructions, Etapes, Objectif final, Restrictions |
| **RACE** | Taches rapides, quotidiennes | Role, Action, Contexte, Resultat attendu |
| **CREATE** | Travail creatif avec exemples | Personnage, Requete, Exemples, Ajustements, Type, Extras |
| **APE** | Brainstorming, taches simples | Action, But, Resultat attendu |
| **STOKE** | Expertise de domaine, technique | Situation, Tache, Objectif, Connaissances, Exemples |

Cliquez sur un framework pour l'appliquer.

### Appliquer un framework

Les blocs actuels sont remplaces par la structure du framework choisi. Chaque bloc contient un titre `## Section` que vous completez avec votre contenu.

> **Attention** : appliquer un framework remplace vos blocs actuels. Sauvegardez une version d'abord si vous ne voulez pas perdre votre travail.

### Creer votre propre framework

Vous n'etes pas limite aux 6 frameworks integres. Creez vos propres structures reutilisables.

#### Methode 1 : Creation manuelle

1. Cliquez le bouton **Creer** en haut de l'onglet Frameworks
2. **Etape 1** — Donnez un nom (ex: "Mon framework SEO") et une description optionnelle
3. **Etape 2** — Composez vos blocs :
   - Choisissez le type de chaque bloc (Role, Contexte, Tache, etc.)
   - Remplissez le contenu pre-rempli (ce texte servira de modele quand vous appliquerez le framework)
   - Ajoutez autant de blocs que necessaire avec les boutons `+ Role`, `+ Tache`, etc.
   - Supprimez un bloc avec l'icone corbeille
4. Cliquez **Creer le framework**

#### Methode 2 : Depuis le prompt actuel

Vous avez construit un prompt dont la structure vous plait ? Sauvegardez-la comme framework :

1. Cliquez le bouton **Depuis actuel**
2. Donnez un nom et une description
3. Les blocs de votre prompt actuel sont affiches en apercu
4. Cliquez **Sauvegarder le framework**

Vos blocs actuels (avec leur contenu) deviennent le modele du framework.

### Gerer vos frameworks

Vos frameworks custom apparaissent dans la section **Mes frameworks** (en haut de la liste). Au survol, trois actions :

| Action | Icone | Description |
|--------|-------|-------------|
| **Modifier** | Crayon | Changer le nom, la description ou les blocs |
| **Dupliquer** | Copie | Creer une copie (utile pour faire des variantes) |
| **Supprimer** | Corbeille | Supprimer definitivement |

### Conseils

- **Commencez par un framework integre**, adaptez-le a vos besoins, puis sauvegardez le resultat avec "Depuis actuel"
- **Mettez du contenu utile** dans les blocs-modeles : instructions recurrentes, balises XML, structure de sortie — tout ce que vous reutilisez souvent
- **Creez des frameworks par domaine** : un pour le SEO, un pour le code, un pour l'analyse de donnees, etc.

---

## 7. Le preview en temps reel

### Acceder au preview

Ouvrez le panneau droit et selectionnez l'onglet **Preview**.

### Ce que vous voyez

Le preview montre le **prompt compile final**, c'est-a-dire :
- Seuls les blocs **actifs** (oeil ouvert) sont inclus
- Les blocs sont joints dans l'ordre affiché
- Les `{{variables}}` sont remplacees par leurs valeurs

### Copier le prompt

Cliquez sur le bouton **Copier** en haut du preview. Le prompt est copie dans votre presse-papiers, pret a etre colle dans ChatGPT, Claude, ou tout autre outil.

---

## 8. Tester votre prompt (Playground)

Le Playground vous permet de tester votre prompt contre des modeles d'IA — en **local** (gratuit, via Ollama) ou en **cloud** (APIs payantes) — sans quitter l'application.

### Modeles locaux (Ollama)

Si le serveur Rust (`prompt-ai-server`) est lance et connecte a Ollama, les **modeles locaux apparaissent automatiquement** en vert en haut du Playground. Aucune cle API n'est requise.

Pour utiliser des modeles locaux :
1. Installez Ollama et telechargez des modeles (`ollama pull mistral-small3.1`)
2. Lancez `prompt-ai-server` et activez le serveur
3. Dans le Playground, les modeles apparaissent en vert — cliquez pour les selectionner

> Les modeles locaux sont **gratuits** et vos donnees ne quittent jamais votre reseau.

### Modeles cloud (APIs)

Les modeles cloud s'affichent en dessous des modeles locaux. Pour les utiliser :

1. Cliquez sur l'icone **engrenage** pour ouvrir les parametres
2. Entrez vos cles API :
   - **OpenAI** : commence par `sk-...` (depuis platform.openai.com)
   - **Anthropic** : commence par `sk-ant-...` (depuis console.anthropic.com)
   - **Google** : commence par `AI...` (depuis aistudio.google.com)

> Les cles sont stockees **localement dans votre navigateur**.

### Configurer le serveur local

Dans les parametres (engrenage), la section **Serveur local** permet de :
- Definir l'URL du serveur Rust (defaut `http://localhost:8910`)
- Voir l'indicateur de connexion (vert = connecte, rouge = inaccessible)
- Voir le nombre de modeles Ollama detectes

Si le serveur est sur une autre machine : `http://192.168.1.100:8910`

### Choisir un modele

Les modeles s'affichent sous forme de boutons :
- **Vert** = modele local (Ollama, gratuit)
- **Violet** = modele cloud (API, payant)
- Cliquez pour **selectionner**, cliquez a nouveau pour **deselectionner**
- Vous pouvez **mixer local et cloud** pour comparer

### Ajuster les parametres

Ouvrez les parametres (icone engrenage) :

- **Temperature** (0.0 - 2.0)
  - 0.0-0.3 : Reponses precises, repetables (ideal pour code, donnees)
  - 0.4-0.6 : Equilibre coherence/creativite
  - 0.7-0.9 : Plus creatif (ideal pour redaction, brainstorming)
  - 1.0+ : Tres creatif, risque d'incoherence

- **Max tokens** (256 - 8192)
  - Limite la longueur de la reponse
  - 2048 : defaut, adapte a la plupart des usages

### Lancer l'execution

Cliquez sur le bouton **Executer**. Le prompt compile est envoye au(x) modele(s) selectionne(s).

### Lire les resultats

Chaque resultat affiche :
- **Nom du modele** en vert (local) ou violet (cloud)
- **Metriques** : latence (ms), tokens utilises, cout estime ou **"gratuit"** pour les modeles locaux
- **Reponse** du modele

En cas d'erreur (cle manquante, serveur inaccessible, etc.), le message s'affiche en rouge.

---

## 9. Comparer plusieurs modeles

### Pourquoi comparer ?

Un meme prompt peut produire des resultats tres differents selon le modele. La comparaison vous aide a :
- Choisir le meilleur modele pour votre cas d'usage
- Evaluer le rapport qualite/prix
- Verifier la coherence des reponses

### Comment faire

1. Dans le Playground, selectionnez **2 ou 3 modeles** (ex: GPT-4o Mini + Claude Sonnet + Gemini Flash)
2. Cliquez **Executer**
3. Les resultats s'affichent **cote a cote** en grille

### Que comparer

| Critere | Ou le voir |
|---------|-----------|
| Qualite de la reponse | Lire et comparer le texte |
| Vitesse | Latence en ms (en haut a droite de chaque resultat) |
| Cout | Estimation en $ (en haut a droite) |
| Tokens utilises | Nombre total input + output |

### Conseil

Commencez par les modeles les moins chers (GPT-4o Mini, Gemini Flash, Claude Haiku) pour iterer rapidement. Une fois satisfait du prompt, testez avec un modele plus puissant pour la production.

---

## 10. Optimiser votre prompt avec l'IA

### Acceder a l'optimiseur

Ouvrez le panneau droit et selectionnez l'onglet **IA**.

### Fonctionnement

1. Cliquez sur **Ameliorer ce prompt**
2. L'application envoie votre prompt compile a un modele d'IA avec un meta-prompt qui demande une optimisation
3. Le modele analyse votre prompt et propose une version amelioree

### Quel modele est utilise ?

L'optimiseur utilise automatiquement le premier modele disponible dans cet ordre de preference :
1. **Claude Sonnet 4.6** (si cle Anthropic configuree)
2. **GPT-4o** (si cle OpenAI configuree)
3. **Gemini 2.5 Flash** (si cle Google configuree)

> Vous devez avoir configure au moins une cle API dans le Playground.

### Appliquer l'optimisation

Si la version proposee vous convient, cliquez **Appliquer le prompt optimise**. Le contenu sera injecte dans votre bloc Tache.

### Conseil

L'optimisation IA est un point de depart, pas une fin. Relisez toujours la version proposee et ajustez selon vos besoins specifiques.

---

## 11. Valider votre prompt (Linting)

### Acceder au linting

Ouvrez le panneau droit et selectionnez l'onglet **Lint**.

### Les verifications effectuees

| Icone | Niveau | Signification |
|-------|--------|--------------|
| Cercle rouge | **Erreur** | Probleme bloquant (ex: aucun bloc actif) |
| Triangle orange | **Warning** | Probleme potentiel (blocs vides, variables non resolues) |
| Cercle bleu | **Info** | Conseil d'amelioration |
| Cercle vert | **OK** | Le prompt semble bien structure |

### Liste des regles

1. **Aucun bloc actif** → Ajoutez au moins un bloc avec du contenu
2. **Blocs vides** → Remplissez ou desactivez les blocs sans contenu
3. **Pas de bloc Tache** → Definissez toujours une directive claire
4. **Variables non resolues** → Renseignez les valeurs dans le panneau Variables
5. **Prompt trop court** → Ajoutez du contexte pour de meilleurs resultats
6. **Prompt trop long** → Certains modeles ont des limites plus basses
7. **Pas d'exemples** → Pour les prompts longs, les exemples few-shot ameliorent la qualite
8. **Instructions negatives** → Preferez "fais X" a "ne fais pas Y"

---

## 12. Organiser vos prompts par projet

### Le concept : Projets et Prompts

Dans un vrai workflow, un projet necessite rarement un seul prompt. Prompt IDE vous permet de **regrouper vos prompts par projet** (dossier) :

```
📂 Mon App E-commerce               ← Projet (dossier)
  📄 System prompt backend            ← Prompt
  📄 Generation fiches produit        ← Prompt
  📄 Analyse des reviews              ← Prompt
📂 Blog SEO                         ← Autre projet
  📄 Redaction articles
  📄 Meta-descriptions
── Prompts libres ──                 ← Prompts sans projet
  📄 Test rapide
```

### Acceder a la bibliotheque

Ouvrez le panneau gauche (onglet **Bibliotheque**).

### Creer un projet (dossier)

1. Cliquez sur l'icone **dossier+** en haut de la bibliotheque
2. Tapez le nom du projet (ex: "Mon App E-commerce")
3. Appuyez **Entree** ou cliquez **Creer**

Le projet apparait avec une pastille de couleur aleatoire et se deplie automatiquement.

### Creer un prompt dans un projet

Deux methodes :
- **Survolez** le nom du projet → cliquez le bouton **+** qui apparait
- **Clic droit** sur le projet → **Nouveau prompt ici**

Le prompt est automatiquement range dans ce projet.

### Creer un prompt libre (hors projet)

Cliquez sur l'icone **+** en haut de la bibliotheque (a cote du dossier+). Ce prompt n'appartiendra a aucun projet et sera affiche dans la section "Prompts libres" en bas.

### Deplier / Replier un projet

Cliquez sur le nom du projet pour le deplier ou le replier. Le chevron (>) indique l'etat. Le projet contenant le prompt actif est deplie automatiquement.

### Naviguer entre les prompts

Cliquez sur un prompt dans l'arborescence pour le charger. Le prompt actif est surligne avec une bordure violette a gauche.

### Deplacer un prompt vers un projet

1. **Clic droit** sur le prompt
2. Dans le menu, section **Deplacer vers**, choisissez le projet cible
3. Ou choisissez **Prompt libre (hors projet)** pour le retirer d'un projet

Vous pouvez deplacer n'importe quel prompt (actif ou non) vers n'importe quel projet.

### Renommer un projet

**Clic droit** sur le projet → **Renommer**. Modifiez le nom et appuyez Entree.

### Renommer un prompt

Cliquez sur le nom du prompt dans la **barre d'en-tete** (en haut) et tapez le nouveau nom. Appuyez Entree pour valider.

### Supprimer un projet

**Clic droit** sur le projet → **Supprimer le projet**. Les prompts du projet ne sont **pas** supprimes : ils deviennent des "prompts libres".

### Supprimer un prompt

Deux methodes :
- **Survolez** le prompt → cliquez l'icone **corbeille**
- **Clic droit** → **Supprimer le prompt**

La suppression efface aussi toutes les versions et executions associees au prompt.

### Rechercher

La barre de recherche en haut filtre a la fois les **projets** et les **prompts** par nom.

### Informations affichees

Pour chaque prompt, vous voyez :
- **Nom** du prompt
- **Date** de derniere modification (relative : "il y a 5min", "il y a 2h", "3 avr.")
- **Framework** utilise (badge colore, si applicable)
- **Tags** (le premier tag)

Pour chaque projet :
- **Pastille de couleur**
- **Nom** du projet
- **Nombre de prompts** qu'il contient

### Sauvegarde automatique

Vos prompts sont sauvegardes automatiquement toutes les 500ms apres une modification. L'indicateur "Sauvegarde..." apparait brievement dans la barre d'en-tete.

---

## 13. Versionner votre travail

### Pourquoi versionner ?

Le versionnage vous permet de :
- Garder un historique de vos iterations
- Revenir a une version precedente si une modification ne convient pas
- Comparer l'evolution de votre prompt

### Sauvegarder une version

1. Ouvrez le panneau gauche (onglet **Versions**)
2. Tapez un label descriptif (ex: "v1 - brouillon initial", "v2 - ajout exemples")
3. Cliquez **Sauver** ou appuyez Entree

### Consulter une version

Cliquez sur une version dans la liste pour la deplier. Le contenu compile s'affiche en apercu.

### Restaurer une version

Cliquez sur l'icone **fleche circulaire** a droite d'une version. Les blocs et variables du projet actuel sont remplaces par ceux de la version.

> **Conseil** : sauvegardez une version avant d'appliquer un framework ou une optimisation IA, pour pouvoir revenir en arriere.

---

## 14. Exporter votre prompt

### Acceder a l'export

Ouvrez le panneau droit et selectionnez l'onglet **Export**.

### Formats disponibles

| Format | Contenu | Usage |
|--------|---------|-------|
| **Texte brut** (.txt) | Prompt compile uniquement | Copier-coller rapide |
| **Markdown** (.md) | Titre + prompt compile | Documentation, partage |
| **JSON complet** (.json) | Blocs, variables, compile, metadonnees | Sauvegarde, reimportation |
| **OpenAI API** (.json) | Format `messages` pret a l'emploi | Integration API directe |
| **Anthropic API** (.json) | Format `messages` pret a l'emploi | Integration API directe |

### Utiliser l'export API

Les exports **OpenAI API** et **Anthropic API** generent un JSON directement utilisable avec `curl` ou un SDK :

**Exemple OpenAI :**
```json
{
  "model": "gpt-4o",
  "messages": [
    { "role": "user", "content": "Votre prompt compile ici..." }
  ],
  "temperature": 0.7
}
```

**Exemple Anthropic :**
```json
{
  "model": "claude-sonnet-4-6",
  "max_tokens": 2048,
  "messages": [
    { "role": "user", "content": "Votre prompt compile ici..." }
  ]
}
```

---

## 15. Dicter vos prompts (Speech-to-Text)

Plutot que de tout taper au clavier, vous pouvez **dicter vos prompts a la voix**. Le texte transcrit est insere directement dans le bloc de votre choix.

### Activer et configurer le STT

1. Ouvrez le panneau droit, onglet **STT**
2. Choisissez un **fournisseur** :

| Fournisseur | Avantage | Prerequis |
|-------------|----------|-----------|
| **Serveur local (Rust)** | Gratuit, vos donnees restent chez vous | Installer et lancer l'app Rust (voir section 16) |
| **OpenAI Whisper** | Tres precis (~2.5% WER) | Cle API OpenAI (celle du Playground) |
| **Groq Whisper** | Ultra rapide, tres pas cher ($0.0007/min) | Cle API Groq |
| **Deepgram Nova-3** | Excellent en temps reel | Cle API Deepgram |

3. Si vous choisissez **Serveur local**, entrez l'URL du serveur (ex: `http://localhost:8910` ou `http://192.168.1.100:8910` si le serveur est sur une autre machine)
4. Si vous choisissez **Groq** ou **Deepgram**, entrez la cle API dans le champ affiche
5. Choisissez la **langue** (auto-detection par defaut, ou forcez francais, anglais, etc.)

### Dicter dans un bloc

1. **Survolez** un bloc → un bouton **micro** apparait dans le header
2. **Cliquez** sur le micro → l'enregistrement commence (bordure rouge pulsante)
3. **Parlez** normalement
4. **Cliquez** a nouveau sur le micro pour arreter
5. Le texte transcrit est **insere a la fin** du contenu du bloc

### Indicateurs visuels

| Etat | Apparence |
|------|-----------|
| Pret | Icone micro grise (visible au survol) |
| Enregistrement | Icone rouge pulsante + bordure rouge + message "Enregistrement en cours..." |
| Transcription | Icone spinner |
| Erreur | Message rouge sous le header (disparait apres 4 secondes) |

### Conseils pour une bonne dictee

- **Parlez clairement** et a un rythme normal
- **Evitez le bruit de fond** — le navigateur active la suppression de bruit automatiquement
- **Dictez par bloc** plutot que tout d'un coup : un bloc Role, puis un bloc Contexte, etc.
- **Relisez et corrigez** apres la dictee — aucun STT n'est parfait
- Pour les termes techniques, **epeler** peut etre necessaire ou ajoutez-les apres manuellement

### Utiliser un serveur distant

Si vous avez un PC puissant avec un GPU, vous pouvez y installer le serveur STT Rust et y acceder depuis n'importe quel autre appareil du reseau :

1. Lancez `prompt-ai-server` sur le PC puissant
2. Notez son IP locale (ex: `192.168.1.100`)
3. Dans l'app web, configurez l'URL du serveur : `http://192.168.1.100:8910` (dans les parametres du Playground ou de l'onglet STT)
4. **Tout passe par cette unique URL** : la dictee (Whisper) ET les modeles LLM (Ollama)

---

## 16. Installer le serveur local (Prompt AI Server)

Le serveur local est une application desktop qui fait tourner vos modeles d'IA sur votre propre machine. Il gere a la fois la **dictee vocale** (Whisper) et les **modeles de langage** (Ollama). L'app web se connecte avec **une seule URL**.

### Prerequis

- **Rust** installe (rustup.rs)
- **Compilateur C++** : `g++` (Linux), Xcode (macOS), Visual Studio Build Tools (Windows)
- **CMake**
- **Linux uniquement** : `libssl-dev`, `libclang-dev`
- **Ollama** installe separement (pour les LLM) : `curl -fsSL https://ollama.com/install.sh | sh`

### Installation

```bash
cd prompt-stt-server
cargo build --release
```

La premiere compilation prend plusieurs minutes (whisper.cpp est compile depuis les sources).

### Lancement

```bash
./target/release/prompt-ai-server
```

L'interface graphique s'ouvre.

### Utilisation pas a pas

#### 1. Installer des modeles LLM (Ollama)

Dans un terminal (separement de l'app) :

```bash
# Meilleur pour le francais
ollama pull mistral-small3.1

# Meilleur pour le raisonnement / meta-prompting
ollama pull deepseek-r1:32b

# Petit modele pour tests rapides
ollama pull qwen2.5:7b
```

#### 2. Connecter Ollama

Dans l'app, section **Ollama (LLM proxy)** :
- Activez le toggle
- Verifiez que l'URL est correcte (defaut `http://localhost:11434`)
- Cliquez **Tester** — le statut doit passer a "Connecte" avec la liste de vos modeles

#### 3. Telecharger un modele Whisper (STT)

Dans la section **Modeles Whisper**, cliquez **DL** a cote du modele souhaite :

| Modele | Taille | Recommande pour |
|--------|--------|----------------|
| **Tiny** (75 Mo) | CPU faible, Raspberry Pi | Tests rapides |
| **Base** (142 Mo) | CPU standard | Usage quotidien leger |
| **Small** (466 Mo) | CPU correct | **Meilleur compromis** pour la dictee |
| **Medium** (1.5 Go) | CPU puissant ou GPU | Bonne qualite |
| **Large v3 Turbo** (1.5 Go) | GPU 6 Go+ | **Recommande avec GPU** |
| **Large v3** (2.9 Go) | GPU 10 Go+ | Meilleure qualite absolue |

#### 4. Charger le modele Whisper

Selectionnez le modele installe et cliquez **Charger**. Le statut passe a "Pret".

#### 5. Demarrer le serveur

Activez le toggle **Serveur HTTP**. Le statut passe a "En ligne" avec l'URL affichee.

#### 6. Connecter l'app web

Dans l'app web, ouvrez les parametres du **Playground** (engrenage) :
- Dans **Serveur local**, entrez l'URL (ex: `http://localhost:8910` ou `http://192.168.1.100:8910`)
- L'indicateur passe au vert
- Les modeles Ollama apparaissent automatiquement dans le Playground
- La dictee vocale fonctionne dans l'onglet STT avec la meme URL

**Une seule URL configure tout** : LLM + STT.

### Quel modele Whisper choisir ?

| Votre machine | Modele recommande | Qualite attendue |
|---------------|------------------|-----------------|
| Laptop sans GPU | **Small** (466 Mo) | Bonne, quasi temps-reel |
| Desktop avec GTX 1660 / RTX 3060 | **Large v3 Turbo** (1.5 Go) | Tres bonne, rapide |
| Desktop avec RTX 3080+ / 4070+ | **Large v3** (2.9 Go) | Excellente |
| Raspberry Pi / mini PC | **Tiny** (75 Mo) | Basique mais fonctionnelle |

### Quels modeles LLM installer dans Ollama ?

| Votre GPU | Modele recommande | Commande | Utilisation |
|-----------|------------------|----------|-------------|
| Pas de GPU (CPU) | Qwen 2.5-7B | `ollama pull qwen2.5:7b` | Tests, iteration rapide |
| 8-12 Go (RTX 3070) | Qwen 2.5-14B | `ollama pull qwen2.5:14b` | Usage general |
| 16-24 Go (RTX 4090) | Mistral Small 3.1 | `ollama pull mistral-small3.1` | **Meilleur en francais** |
| 16-24 Go (RTX 4090) | DeepSeek R1-32B | `ollama pull deepseek-r1:32b` | Optimisation de prompts |
| 48+ Go (2x GPU) | Llama 3.3-70B | `ollama pull llama3.3:70b` | Meilleure qualite |

---

## 17. Le compteur de tokens

### Ou le trouver

Le compteur est la barre fixe en bas du panneau central.

### Informations affichees

| Element | Signification |
|---------|--------------|
| **# 247 tokens** | Nombre de tokens du prompt compile |
| **~$0.002** | Cout estime (input + 50% output estime) |
| **3/5 blocs** | Blocs actifs sur le total |
| **Barre de progression** | Pourcentage du contexte max du modele utilise |
| **Selecteur de modele** | Change le modele pour l'estimation de cout |

### Code couleur de la barre de contexte

| Couleur | Utilisation | Signification |
|---------|------------|--------------|
| Violet | < 50% | Confortable |
| Orange | 50-80% | Attention a la taille |
| Rouge | > 80% | Risque de depassement |

### Comprendre les tokens

- 1 token ≈ 4 caracteres en anglais, ~3 caracteres en francais
- ~750 mots = ~1000 tokens
- Le cout depend du modele choisi (voir le selecteur)
- L'estimation de cout suppose que la reponse fera ~50% de la taille du prompt

---

## 18. Raccourcis et astuces

### Dans l'editeur de blocs

| Action | Raccourci |
|--------|----------|
| Annuler | `Ctrl+Z` |
| Refaire | `Ctrl+Shift+Z` |
| Auto-completion variables | Taper `{{` |

### Astuces de productivite

1. **Organisez par projet des le depart** : creez un projet (dossier) pour chaque contexte de travail — un projet par application, par client, ou par campagne
2. **Desactivez les blocs pour A/B tester** : gardez un bloc Exemples desactive et comparez les resultats avec et sans
3. **Utilisez les frameworks comme point de depart** : ne partez jamais d'une page blanche, choisissez un framework puis adaptez
4. **Versionnez avant chaque changement majeur** : cela prend 2 secondes et vous sauve en cas de mauvaise direction
5. **Commencez par les modeles pas chers** : iterez avec GPT-4o Mini ou Gemini Flash, finalisez avec un modele premium
6. **Lisez le linting** : les conseils detectent des erreurs courantes qui baissent la qualite de vos prompts
7. **Exportez en JSON complet** : c'est la meilleure facon de sauvegarder ou partager un prompt avec toute sa structure
8. **Clic droit = menu contextuel** : sur les projets et les prompts, le clic droit offre des actions rapides (deplacer, renommer, supprimer)

### Coloration syntaxique

L'editeur colore automatiquement :
- `{{variables}}` en **violet** avec fond leger
- `<balises_xml>` en **bleu**
- `## Titres de section` en **vert gras**
- `// Commentaires` en **gris italique** (non inclus dans le prompt compile si vous les mettez dans des blocs desactives)

---

## 19. Import un prompt

### Pourquoi importer ?

Si un collegue vous envoie un fichier `.json` exporte depuis Prompt IDE, vous pouvez le reimporter en un clic pour retrouver la structure complete du prompt (blocs, variables, nom).

### Comment importer

1. Ouvrez le panneau droit et selectionnez l'onglet **Export**
2. Cliquez sur le bouton **Importer un JSON**
3. Selectionnez un fichier `.json` exporte precedemment depuis Prompt IDE
4. Le prompt est charge immediatement : les blocs, les variables et le nom du prompt sont restaures

### Format attendu

Le fichier JSON doit contenir un tableau `"blocks"` pour etre valide. C'est le format genere automatiquement par l'export **JSON complet** (voir section 14). Si le fichier ne contient pas de tableau `blocks`, l'import echouera avec un message d'erreur.

---

## 20. Reponses en streaming

### Comment ca marche

Les reponses du Playground s'affichent desormais **mot par mot** au fur et a mesure qu'elles sont generees, comme dans ChatGPT. Vous n'avez plus besoin d'attendre la fin de la generation pour commencer a lire.

### Compatibilite par fournisseur

| Fournisseur | Streaming |
|-------------|-----------|
| **OpenAI** (GPT-4o, etc.) | Oui, mot par mot |
| **Modeles locaux** (Ollama) | Oui, mot par mot |
| **Anthropic** (Claude) | Reponse complete affichee d'un coup |
| **Google** (Gemini) | Reponse complete affichee d'un coup |

### Indicateur visuel

Pendant la reception d'une reponse en streaming, un **spinner** s'affiche a cote du nom du modele pour indiquer que la generation est en cours. Il disparait une fois la reponse complete.

---

## 21. Historique des executions

### Acceder a l'historique

Ouvrez le panneau droit et selectionnez l'onglet **Historique** (icone horloge).

### Ce que vous voyez

L'historique affiche tous les resultats de tests passes pour le prompt actuellement ouvert. Chaque entree contient :

| Information | Description |
|-------------|-------------|
| **Modele** | Le modele utilise pour cette execution |
| **Date** | Date et heure de l'execution |
| **Latence** | Temps de reponse en millisecondes |
| **Tokens** | Nombre de tokens utilises |
| **Cout** | Cout estime de l'execution |
| **Apercu** | Debut de la reponse generee |

### Consulter une reponse

Cliquez sur une entree de l'historique pour afficher la **reponse complete** du modele.

### Effacer l'historique

Cliquez sur le bouton **Effacer l'historique** pour supprimer toutes les entrees de l'historique du prompt actuel.

---

## 22. Chainer des prompts (Workflows)

### Le concept

Le chainage permet d'executer **plusieurs prompts a la suite**, ou la sortie d'un prompt alimente automatiquement le suivant. C'est utile pour les workflows complexes en plusieurs etapes (recherche → analyse → redaction, par exemple).

### Acceder au chainage

Ouvrez le panneau droit et selectionnez l'onglet **Chain** (icone maillon).

### Comment chainer

1. **Selectionnez un projet** (workspace) contenant les prompts a chainer
2. Les prompts du projet sont affiches dans l'ordre de creation
3. **Choisissez un modele** qui sera utilise pour toute la chaine
4. Cliquez **Executer la chaine**

### Passage de donnees entre etapes

La sortie de l'etape N est automatiquement disponible comme variable `{{chain_output_N}}` pour l'etape N+1. Par exemple :

- **Etape 1** : "Genere une liste de 5 sujets d'articles" → produit `{{chain_output_1}}`
- **Etape 2** : "A partir de {{chain_output_1}}, redige un plan detaille pour le premier sujet"

### Resultats

Les resultats s'affichent **en ligne** pour chaque etape, avec le numero de l'etape, le nom du prompt et la reponse du modele.

### Gestion des erreurs

Si une etape echoue (erreur API, timeout, etc.), la chaine **s'arrete immediatement** et l'erreur est affichee. Les etapes precedentes restent visibles avec leurs resultats.

---

## 23. Mode Conversation (Chat)

### Acceder au chat

Ouvrez le panneau droit et selectionnez l'onglet **Chat** (icone message).

### Fonctionnement

Le mode Chat offre une interface de conversation multi-tour avec un modele d'IA, directement dans Prompt IDE.

1. **Choisissez un modele** : local (Ollama) ou cloud (OpenAI, Anthropic, Google)
2. **System prompt optionnel** : cliquez sur le bouton pour utiliser votre prompt actuel comme system prompt de la conversation
3. **Tapez votre message** dans le champ en bas et envoyez
4. La reponse du modele s'affiche dans l'interface de chat
5. Continuez la conversation — l'**historique complet** est maintenu et envoye a chaque echange

### Streaming

Les reponses en mode Chat beneficient egalement du streaming (voir section 20) : les mots apparaissent au fur et a mesure pour les modeles compatibles.

### Recommencer

Cliquez sur **Clear conversation** pour effacer l'historique et recommencer une conversation vierge.

---

## 24. Statistiques

### Acceder aux statistiques

Ouvrez le panneau droit et selectionnez l'onglet **Stats** (icone graphique a barres).

### Metriques affichees

| Metrique | Description |
|----------|-------------|
| **Total executions** | Nombre total de tests lances |
| **Total tokens** | Nombre cumule de tokens consommes |
| **Cout total** | Depense cumulee estimee |
| **Latence moyenne** | Temps de reponse moyen |
| **Modele le plus utilise** | Le modele que vous avez le plus sollicite |

### Graphique

Un graphique a barres montre la repartition des executions par modele, pour visualiser rapidement quels modeles vous utilisez le plus.

### Filtrer par periode

Trois filtres sont disponibles :
- **7 derniers jours**
- **30 derniers jours**
- **Tout le temps**

---

## 25. Application mobile (Responsive)

### Utiliser Prompt IDE sur mobile

L'application s'adapte automatiquement aux petits ecrans (moins de 768 pixels de large).

### Ce qui change sur mobile

- Les **panneaux lateraux** (gauche et droit) deviennent des **overlays plein ecran** qui glissent par-dessus le contenu
- Les panneaux sont **replies par defaut** au chargement pour maximiser l'espace d'edition
- Cliquez sur le **fond sombre** (backdrop) pour fermer un panneau ouvert
- Toutes les fonctionnalites restent accessibles : edition, Playground, export, chat, etc.

### Conseil

Sur mobile, travaillez un panneau a la fois : ouvrez le panneau gauche pour naviguer entre vos prompts, fermez-le, editez, puis ouvrez le panneau droit pour tester ou exporter.

---

## 26. Installation hors-ligne (PWA)

### Qu'est-ce qu'une PWA ?

Prompt IDE est une **Progressive Web App** : elle peut etre installee comme une application native depuis votre navigateur, et fonctionne meme hors-ligne.

### Installer l'application

- **Chrome / Edge** : cliquez sur l'icone **Installer** qui apparait dans la barre d'adresse (ou dans le menu ⋮ → "Installer Prompt IDE")
- **Safari (iOS)** : appuyez sur le bouton Partager → "Sur l'ecran d'accueil"

### Fonctionnement hors-ligne

Une fois installee, l'application met en cache toutes les **ressources statiques** (HTML, CSS, JavaScript). Vous pouvez l'ouvrir sans connexion internet.

Les fonctionnalites suivantes marchent hors-ligne :
- Edition de blocs et de variables
- Versionnage
- Export
- Frameworks

Les fonctionnalites necessitant une connexion (Playground cloud, optimisation IA) restent indisponibles hors-ligne, sauf si vous utilisez un serveur local (Ollama).

### Stockage des donnees

Toutes vos donnees sont stockees localement dans **IndexedDB** (la base de donnees integree au navigateur). Rien n'est envoye a un serveur distant.

---

## 27. Deployer avec Docker

### Demarrage rapide

```
docker compose up -d
```

L'application est accessible sur **http://localhost:3000**.

### Changer le port

Modifiez le fichier `docker-compose.yml` pour changer le port expose. Par exemple, pour utiliser le port 8080 :

```yaml
ports:
  - "8080:80"
```

### Deployer sur un VPS ou NAS

1. Copiez les fichiers du projet sur votre serveur
2. Lancez `docker compose up -d`
3. Accedez a l'application via l'IP de votre serveur (ex: `http://192.168.1.50:3000`)

### Taille de l'image

L'image Docker pese environ **97 Mo**.

---

## 28. FAQ et depannage

### Ou sont stockees mes donnees ?

Toutes vos donnees (projets, versions, executions) sont dans la base **IndexedDB** de votre navigateur. Elles restent sur votre machine et ne sont jamais envoyees a un serveur.

**Attention** : vider le cache/donnees du navigateur efface aussi vos projets. Utilisez l'export JSON complet pour faire des sauvegardes.

### Mes cles API sont-elles securisees ?

Elles sont stockees dans le `localStorage` de votre navigateur. C'est adapte pour un usage personnel, mais :
- Ne partagez pas l'acces a votre navigateur
- En entreprise, preferez un backend proxy
- Les cles ne sont jamais envoyees ailleurs qu'aux APIs officielles (OpenAI, Anthropic, Google)

### L'execution renvoie une erreur

| Erreur | Cause probable | Solution |
|--------|---------------|----------|
| "Cle API manquante" | Pas de cle configuree | Ouvrez les parametres du Playground |
| "401 Unauthorized" | Cle API invalide ou expiree | Verifiez votre cle sur le site du fournisseur |
| "429 Too Many Requests" | Quota depasse | Attendez ou augmentez votre plan |
| "CORS error" | Restriction navigateur (Anthropic) | Normal pour Anthropic en direct browser — cela fonctionne grace au header special |
| Pas de reponse | Prompt trop long pour le modele | Reduisez la taille ou choisissez un modele avec plus de contexte |

### Le compteur de tokens est-il precis ?

Le compteur utilise le tokenizer GPT (gpt-tokenizer). Il est precis pour les modeles OpenAI. Pour Claude et Gemini, c'est une **approximation** (+/- 5-10%) car chaque fournisseur a son propre tokenizer.

### Puis-je utiliser l'app hors-ligne ?

Oui, l'app est une PWA installable (voir section 26). Les ressources statiques sont mises en cache :
- **L'edition, le versionnage, l'export** fonctionnent hors-ligne
- **Le Playground et l'Optimisation IA** necessitent une connexion (appels API) sauf si vous utilisez un serveur local (Ollama)

### Le micro ne fonctionne pas

| Probleme | Solution |
|----------|----------|
| "Impossible d'acceder au micro" | Autorisez l'acces au micro dans les parametres du navigateur |
| "Serveur local inaccessible" | Verifiez que l'app Rust est lancee et le serveur active |
| Texte vide apres transcription | L'audio etait trop court ou trop bruite — parlez plus fort |
| Transcription tres lente | Utilisez un modele plus petit (tiny/base) ou passez sur une API cloud |

### Quelle API STT choisir ?

- **Gratuit + vie privee** : serveur local (Rust) avec modele Small
- **Meilleure qualite** : OpenAI gpt-4o-mini-transcribe
- **Plus rapide et pas cher** : Groq Whisper ($0.04/h)
- **Meilleur en temps reel** : Deepgram Nova-3

### Les modeles locaux (Ollama) n'apparaissent pas dans le Playground

1. Verifiez qu'Ollama est lance (`ollama serve` dans un terminal)
2. Verifiez que `prompt-ai-server` est lance avec le serveur active
3. Verifiez que l'URL du serveur local est correcte dans les parametres du Playground
4. L'indicateur de connexion doit etre vert
5. Si Ollama est sur une autre machine, exposez-le : `OLLAMA_HOST=0.0.0.0:11434 ollama serve`

### Puis-je utiliser l'app 100% localement sans aucune API cloud ?

Oui. Avec `prompt-ai-server` + Ollama, tout fonctionne sans connexion internet et sans cle API :
- **Playground** : modeles Ollama (Mistral, Qwen, Llama, etc.)
- **Optimisation IA** : utilisera le premier modele local disponible
- **Dictee vocale** : Whisper local
- **Edition, frameworks, export, etc.** : deja 100% local

### Que se passe-t-il si je supprime un projet ?

Les prompts du projet ne sont **pas supprimes**. Ils deviennent des "prompts libres" visibles en bas de la bibliotheque. Seul le dossier disparait.

### Comment partager un prompt avec un collegue ?

1. Exportez en **JSON complet** (onglet Export)
2. Envoyez le fichier .json a votre collegue
3. Il ouvre l'onglet **Export** et clique **Importer un JSON** pour charger le fichier (voir section 19)

### L'application est-elle compatible mobile ?

Oui. L'application est entierement responsive (voir section 25). Sur les ecrans de moins de 768 pixels, les panneaux lateraux deviennent des overlays plein ecran. Toutes les fonctionnalites sont accessibles sur mobile.

### Comment installer l'app sur mon telephone ?

Prompt IDE est une PWA (Progressive Web App) installable depuis votre navigateur mobile :
- **Android (Chrome)** : ouvrez l'app dans Chrome, puis appuyez sur le menu ⋮ → "Installer l'application" ou utilisez la banniere d'installation qui apparait automatiquement
- **iPhone/iPad (Safari)** : ouvrez l'app dans Safari, appuyez sur le bouton Partager → "Sur l'ecran d'accueil"

L'app se comporte ensuite comme une application native. Voir la section 26 pour plus de details.

### Qu'est-ce que le chainage de prompts ?

Le chainage (ou workflows) permet d'executer plusieurs prompts a la suite, ou la sortie d'un prompt alimente le suivant via la variable `{{chain_output_N}}`. C'est utile pour decomposer une tache complexe en plusieurs etapes. Voir la section 22 pour le guide complet.

---

## Glossaire

| Terme | Definition |
|-------|-----------|
| **Projet (Workspace)** | Dossier qui regroupe plusieurs prompts lies a un meme contexte de travail |
| **Prompt** | Un ensemble de blocs formant une instruction complete pour un modele d'IA |
| **Prompt libre** | Prompt non range dans un projet, affiche dans la section "Prompts libres" |
| **Bloc** | Unite de contenu dans l'editeur (Role, Contexte, etc.) |
| **Prompt compile** | Le texte final envoye au modele, apres assemblage des blocs actifs et remplacement des variables |
| **Token** | Unite de texte utilisee par les modeles (~4 caracteres) |
| **Few-shot** | Technique consistant a fournir des exemples d'entree/sortie pour guider le modele |
| **Framework integre** | Structure pre-definie livree avec l'app (CO-STAR, RISEN, RACE, CREATE, APE, STOKE) |
| **Framework custom** | Structure personnalisee creee par l'utilisateur et reutilisable |
| **Temperature** | Parametre controlant la creativite du modele (0 = deterministe, 2 = tres creatif) |
| **Top-p** | Parametre limitant les tokens consideres par probabilite |
| **Linting** | Verification automatique de la qualite et structure du prompt |
| **Meta-prompt** | Technique utilisant l'IA pour ameliorer un prompt existant |
| **Context window** | Taille maximale (en tokens) qu'un modele peut traiter en une fois |
| **STT (Speech-to-Text)** | Technologie de reconnaissance vocale qui convertit la parole en texte |
| **Whisper** | Modele de reconnaissance vocale d'OpenAI, disponible en API et en local |
| **Prompt AI Server** | L'application Rust qui sert de hub local pour Whisper (STT) + Ollama (LLM) |
| **Ollama** | Gestionnaire de modeles LLM locaux, expose une API compatible OpenAI |
| **Modele local** | Modele d'IA qui tourne sur votre machine via Ollama (gratuit, pas de cle API) |
| **Proxy** | Le serveur Rust relaie les requetes LLM vers Ollama — l'app web ne voit qu'une seule URL |
| **Streaming** | Mode d'affichage ou la reponse du modele apparait mot par mot au fur et a mesure de la generation |
| **Chainage (Workflow)** | Execution sequentielle de plusieurs prompts ou la sortie de chaque etape alimente la suivante |
| **PWA (Progressive Web App)** | Application web installable sur un appareil et fonctionnant hors-ligne grace a un Service Worker |
| **Analytics (Statistiques)** | Tableau de bord montrant les metriques d'utilisation : executions, tokens, couts, latence |
| **IndexedDB** | Base de donnees integree au navigateur, utilisee pour le stockage local |

---

> **Besoin d'aide ?** Consultez la [Documentation technique](./DOCUMENTATION.md) pour les details d'architecture et d'API.
