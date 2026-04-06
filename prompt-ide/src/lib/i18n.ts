import { createContext, useContext } from 'react';

export type Lang = 'fr' | 'en';

const LANG_KEY = 'prompt-ide-lang';

export function getLang(): Lang {
  return (localStorage.getItem(LANG_KEY) as Lang) || 'fr';
}

export function setLang(lang: Lang): void {
  localStorage.setItem(LANG_KEY, lang);
}

// --- Translation keys ---

export const translations = {
  // Header
  'app.title': { fr: 'Inkwell', en: 'Inkwell' },
  'app.saving': { fr: 'Sauvegarde...', en: 'Saving...' },

  // Left tabs
  'tab.library': { fr: 'Bibliotheque', en: 'Library' },
  'tab.frameworks': { fr: 'Frameworks', en: 'Frameworks' },
  'tab.versions': { fr: 'Versions', en: 'Versions' },

  // Right tabs
  'tab.preview': { fr: 'Preview', en: 'Preview' },
  'tab.playground': { fr: 'Playground', en: 'Playground' },
  'tab.stt': { fr: 'STT', en: 'STT' },
  'tab.optimize': { fr: 'IA', en: 'AI' },
  'tab.lint': { fr: 'Lint', en: 'Lint' },
  'tab.history': { fr: 'Historique', en: 'History' },
  'tab.export': { fr: 'Export', en: 'Export' },

  // Block types
  'block.role': { fr: 'Role / Persona', en: 'Role / Persona' },
  'block.context': { fr: 'Contexte', en: 'Context' },
  'block.task': { fr: 'Tache / Directive', en: 'Task / Directive' },
  'block.examples': { fr: 'Exemples (Few-shot)', en: 'Examples (Few-shot)' },
  'block.constraints': { fr: 'Contraintes', en: 'Constraints' },
  'block.format': { fr: 'Format de sortie', en: 'Output format' },

  // Block placeholders
  'placeholder.role': { fr: 'Tu es un expert en...', en: 'You are an expert in...' },
  'placeholder.context': { fr: 'Informations de fond, donnees, contexte de la tache...', en: 'Background information, data, task context...' },
  'placeholder.task': { fr: 'Redige, Analyse, Compare, Genere...', en: 'Write, Analyze, Compare, Generate...' },
  'placeholder.examples': { fr: '<example>\nInput: ...\nOutput: ...\n</example>', en: '<example>\nInput: ...\nOutput: ...\n</example>' },
  'placeholder.constraints': { fr: '- Maximum 500 mots\n- Ton professionnel\n- Format liste', en: '- Maximum 500 words\n- Professional tone\n- List format' },
  'placeholder.format': { fr: 'JSON, Markdown, liste numerotee, tableau...', en: 'JSON, Markdown, numbered list, table...' },

  // Block actions
  'block.disable': { fr: 'Desactiver', en: 'Disable' },
  'block.enable': { fr: 'Activer', en: 'Enable' },
  'block.delete': { fr: 'Supprimer', en: 'Delete' },
  'block.dictate': { fr: 'Dicter (Speech-to-Text)', en: 'Dictate (Speech-to-Text)' },
  'block.stopDictate': { fr: 'Arreter et transcrire', en: 'Stop and transcribe' },
  'block.transcribing': { fr: 'Transcription...', en: 'Transcribing...' },
  'block.recording': { fr: 'Enregistrement en cours... Cliquez sur le micro pour arreter.', en: 'Recording... Click the microphone to stop.' },
  'block.add': { fr: 'Ajouter un bloc', en: 'Add a block' },

  // Variables
  'variables.title': { fr: 'Variables', en: 'Variables' },
  'variables.use': { fr: 'Utilisez', en: 'Use' },
  'variables.hint': { fr: 'dans vos blocs pour creer des variables.', en: 'in your blocks to create variables.' },
  'variables.valueFor': { fr: 'Valeur pour', en: 'Value for' },

  // Token counter
  'counter.chars': { fr: 'car.', en: 'chars' },
  'counter.words': { fr: 'mots', en: 'words' },
  'counter.lines': { fr: 'lignes', en: 'lines' },
  'counter.tokens': { fr: 'tokens', en: 'tokens' },
  'counter.blocks': { fr: 'blocs', en: 'blocks' },
  'counter.unresolvedVars': { fr: 'var. non resolue(s)', en: 'unresolved var(s)' },

  // Preview
  'preview.title': { fr: 'Prompt compile', en: 'Compiled prompt' },
  'preview.copy': { fr: 'Copier', en: 'Copy' },
  'preview.copied': { fr: 'Copie!', en: 'Copied!' },
  'preview.empty': { fr: 'Commencez a ecrire dans les blocs pour voir le prompt compile...', en: 'Start writing in the blocks to see the compiled prompt...' },

  // Playground
  'playground.execute': { fr: 'Executer', en: 'Execute' },
  'playground.executing': { fr: 'Execution...', en: 'Executing...' },
  'playground.selectModels': { fr: 'Selectionnez un ou plusieurs modeles et cliquez sur Executer', en: 'Select one or more models and click Execute' },
  'playground.localModels': { fr: 'Modeles locaux (Ollama)', en: 'Local models (Ollama)' },
  'playground.cloudModels': { fr: 'Modeles cloud (API)', en: 'Cloud models (API)' },
  'playground.localServer': { fr: 'Serveur local (STT + LLM)', en: 'Local server (STT + LLM)' },
  'playground.connected': { fr: 'Connecte', en: 'Connected' },
  'playground.notConnected': { fr: 'Non connecte — lancez inkwell-server', en: 'Not connected — launch inkwell-server' },
  'playground.modelsDetected': { fr: 'modele(s) Ollama detecte(s)', en: 'Ollama model(s) detected' },
  'playground.temperature': { fr: 'Temperature', en: 'Temperature' },
  'playground.maxTokens': { fr: 'Max tokens', en: 'Max tokens' },
  'playground.apiKeys': { fr: 'Cles API (cloud)', en: 'API keys (cloud)' },
  'playground.free': { fr: 'gratuit', en: 'free' },
  'playground.missingKey': { fr: 'Cle API manquante.', en: 'Missing API key.' },
  'playground.streaming': { fr: 'Streaming...', en: 'Streaming...' },

  // History
  'history.title': { fr: 'Historique', en: 'History' },
  'history.empty': { fr: 'Aucune execution', en: 'No executions' },
  'history.clear': { fr: 'Effacer l\'historique', en: 'Clear history' },

  // Library
  'library.search': { fr: 'Rechercher...', en: 'Search...' },
  'library.newWorkspace': { fr: 'Nouveau projet (dossier)', en: 'New project (folder)' },
  'library.newPrompt': { fr: 'Nouveau prompt libre', en: 'New free prompt' },
  'library.workspaceName': { fr: 'Nom du projet...', en: 'Project name...' },
  'library.create': { fr: 'Creer', en: 'Create' },
  'library.freePrompts': { fr: 'Prompts libres', en: 'Free prompts' },
  'library.empty': { fr: 'Rien ici encore', en: 'Nothing here yet' },
  'library.emptyHint': { fr: 'Creez un projet (dossier) pour organiser vos prompts, ou un prompt libre.', en: 'Create a project (folder) to organize your prompts, or a free prompt.' },
  'library.noResults': { fr: 'Aucun resultat', en: 'No results' },
  'library.noPrompts': { fr: 'Aucun prompt — cliquez + pour en creer', en: 'No prompts — click + to create one' },
  'library.newPromptHere': { fr: 'Nouveau prompt ici', en: 'New prompt here' },
  'library.rename': { fr: 'Renommer', en: 'Rename' },
  'library.changeColor': { fr: 'Changer la couleur', en: 'Change color' },
  'library.deleteWorkspace': { fr: 'Supprimer le projet', en: 'Delete project' },
  'library.moveTo': { fr: 'Deplacer vers', en: 'Move to' },
  'library.freePromptOption': { fr: 'Prompt libre (hors projet)', en: 'Free prompt (no project)' },
  'library.duplicate': { fr: 'Dupliquer', en: 'Duplicate' },
  'library.deletePrompt': { fr: 'Supprimer le prompt', en: 'Delete prompt' },
  'library.newPromptInProject': { fr: 'Nouveau prompt dans ce projet', en: 'New prompt in this project' },

  // Versions
  'versions.title': { fr: 'Versions', en: 'Versions' },
  'versions.label': { fr: 'Label de version...', en: 'Version label...' },
  'versions.save': { fr: 'Sauver', en: 'Save' },
  'versions.empty': { fr: 'Aucune version sauvegardee', en: 'No saved versions' },
  'versions.restore': { fr: 'Restaurer cette version', en: 'Restore this version' },
  'versions.compare': { fr: 'Comparer', en: 'Compare' },
  'versions.current': { fr: 'Actuel', en: 'Current' },
  'versions.version': { fr: 'Version', en: 'Version' },

  // Frameworks
  'frameworks.title': { fr: 'Frameworks', en: 'Frameworks' },
  'frameworks.create': { fr: 'Creer', en: 'Create' },
  'frameworks.fromCurrent': { fr: 'Depuis actuel', en: 'From current' },
  'frameworks.myFrameworks': { fr: 'Mes frameworks', en: 'My frameworks' },
  'frameworks.builtIn': { fr: 'Frameworks integres', en: 'Built-in frameworks' },
  'frameworks.modify': { fr: 'Modifier', en: 'Edit' },
  'frameworks.duplicate': { fr: 'Dupliquer', en: 'Duplicate' },
  'frameworks.delete': { fr: 'Supprimer', en: 'Delete' },
  'frameworks.newFramework': { fr: 'Nouveau framework', en: 'New framework' },
  'frameworks.editFramework': { fr: 'Modifier le framework', en: 'Edit framework' },
  'frameworks.saveAsFw': { fr: 'Sauvegarder comme framework', en: 'Save as framework' },
  'frameworks.name': { fr: 'Nom du framework *', en: 'Framework name *' },
  'frameworks.namePlaceholder': { fr: 'Ex: Mon framework SEO', en: 'Ex: My SEO framework' },
  'frameworks.description': { fr: 'Description', en: 'Description' },
  'frameworks.descPlaceholder': { fr: 'Courte description...', en: 'Short description...' },
  'frameworks.blocksToSave': { fr: 'Blocs qui seront sauvegardes :', en: 'Blocks that will be saved:' },
  'frameworks.saveFramework': { fr: 'Sauvegarder le framework', en: 'Save framework' },
  'frameworks.nextBlocks': { fr: 'Suivant : definir les blocs', en: 'Next: define blocks' },
  'frameworks.blocksHint': { fr: 'Definissez les blocs de votre framework. Le contenu servira de modele pre-rempli.', en: 'Define your framework blocks. The content will serve as a pre-filled template.' },
  'frameworks.createFramework': { fr: 'Creer le framework', en: 'Create framework' },
  'frameworks.saveChanges': { fr: 'Enregistrer les modifications', en: 'Save changes' },
  'frameworks.stepInfo': { fr: '1. Infos', en: '1. Info' },
  'frameworks.stepBlocks': { fr: '2. Blocs', en: '2. Blocks' },

  // Import
  'import.title': { fr: 'Importer', en: 'Import' },
  'import.button': { fr: 'Importer un JSON', en: 'Import a JSON' },
  'import.success': { fr: 'Prompt importe !', en: 'Prompt imported!' },
  'import.error': { fr: 'Fichier JSON invalide.', en: 'Invalid JSON file.' },

  // Export
  'export.title': { fr: 'Exporter', en: 'Export' },
  'export.txt': { fr: 'Texte brut', en: 'Plain text' },
  'export.md': { fr: 'Markdown', en: 'Markdown' },
  'export.json': { fr: 'JSON (complet)', en: 'JSON (full)' },
  'export.jsonDesc': { fr: 'Blocs + variables + compile', en: 'Blocks + variables + compiled' },
  'export.openai': { fr: 'OpenAI API', en: 'OpenAI API' },
  'export.anthropic': { fr: 'Anthropic API', en: 'Anthropic API' },
  'export.apiDesc': { fr: 'Format messages API', en: 'API messages format' },

  // Optimizer
  'optimizer.title': { fr: 'Optimisation IA', en: 'AI Optimization' },
  'optimizer.improve': { fr: 'Ameliorer ce prompt', en: 'Improve this prompt' },
  'optimizer.improving': { fr: 'Optimisation...', en: 'Optimizing...' },
  'optimizer.apply': { fr: 'Appliquer le prompt optimise (remplace le bloc Tache)', en: 'Apply optimized prompt (replaces Task block)' },
  'optimizer.needKey': { fr: 'Configurez au moins une cle API (OpenAI, Anthropic ou Google) dans le Playground.', en: 'Configure at least one API key (OpenAI, Anthropic or Google) in the Playground.' },

  // Linting
  'lint.title': { fr: 'Validation', en: 'Validation' },
  'lint.noBlocks': { fr: 'Aucun bloc actif. Ajoutez au moins un bloc avec du contenu.', en: 'No active blocks. Add at least one block with content.' },
  'lint.emptyBlocks': { fr: 'bloc(s) actif(s) sans contenu.', en: 'active block(s) without content.' },
  'lint.noTask': { fr: 'Pas de bloc Tache. Definissez une directive claire.', en: 'No Task block. Define a clear directive.' },
  'lint.unresolvedVars': { fr: 'Variable(s) non resolue(s):', en: 'Unresolved variable(s):' },
  'lint.tooShort': { fr: 'Le prompt est tres court. Ajoutez du contexte pour de meilleurs resultats.', en: 'The prompt is very short. Add context for better results.' },
  'lint.tooLong': { fr: 'Le prompt utilise {n} tokens. Certains modeles ont des limites plus basses.', en: 'The prompt uses {n} tokens. Some models have lower limits.' },
  'lint.noExamples': { fr: 'Conseil: ajouter des exemples (few-shot) peut ameliorer la qualite des reponses.', en: 'Tip: adding examples (few-shot) can improve response quality.' },
  'lint.negativeInstructions': { fr: 'Instructions negatives detectees. Preferez les formulations positives ("fais X" au lieu de "ne fais pas Y").', en: 'Negative instructions detected. Prefer positive phrasing ("do X" instead of "don\'t do Y").' },
  'lint.allGood': { fr: 'Le prompt semble bien structure.', en: 'The prompt looks well structured.' },

  // STT
  'stt.title': { fr: 'Speech-to-Text', en: 'Speech-to-Text' },
  'stt.provider': { fr: 'Fournisseur', en: 'Provider' },
  'stt.local': { fr: 'Serveur local (Rust)', en: 'Local server (Rust)' },
  'stt.localDesc': { fr: 'Votre propre serveur STT', en: 'Your own STT server' },
  'stt.openaiDesc': { fr: 'gpt-4o-mini-transcribe', en: 'gpt-4o-mini-transcribe' },
  'stt.groqDesc': { fr: 'Whisper v3-turbo, ultra rapide', en: 'Whisper v3-turbo, ultra fast' },
  'stt.deepgramDesc': { fr: 'Temps reel, 36+ langues', en: 'Real-time, 36+ languages' },
  'stt.serverUrl': { fr: 'URL du serveur local', en: 'Local server URL' },
  'stt.connected': { fr: 'Connecte au serveur', en: 'Connected to server' },
  'stt.disconnected': { fr: 'Serveur inaccessible', en: 'Server unreachable' },
  'stt.checking': { fr: 'Verification...', en: 'Checking...' },
  'stt.language': { fr: 'Langue', en: 'Language' },
  'stt.hint': { fr: "Cliquez sur l'icone micro dans un bloc pour dicter. Maintenez pour enregistrer, relachez pour transcrire.", en: 'Click the microphone icon in a block to dictate. Hold to record, release to transcribe.' },
  'stt.useOpenaiKey': { fr: 'Utilise la cle OpenAI configuree dans le Playground.', en: 'Uses the OpenAI key configured in the Playground.' },

  // Languages
  'lang.auto': { fr: 'Auto-detection', en: 'Auto-detect' },
  'lang.fr': { fr: 'Francais', en: 'French' },
  'lang.en': { fr: 'English', en: 'English' },
  'lang.es': { fr: 'Espanol', en: 'Spanish' },
  'lang.de': { fr: 'Deutsch', en: 'German' },
  'lang.it': { fr: 'Italiano', en: 'Italian' },
  'lang.pt': { fr: 'Portugues', en: 'Portuguese' },
  'lang.ja': { fr: 'Japonais', en: 'Japanese' },
  'lang.zh': { fr: 'Chinois', en: 'Chinese' },
  'lang.ko': { fr: 'Coreen', en: 'Korean' },
  'lang.ru': { fr: 'Russe', en: 'Russian' },
  'lang.ar': { fr: 'Arabe', en: 'Arabic' },
  'lang.nl': { fr: 'Neerlandais', en: 'Dutch' },

  // Auth
  'auth.login': { fr: 'Connexion', en: 'Sign in' },
  'auth.register': { fr: 'Inscription', en: 'Sign up' },
  'auth.email': { fr: 'Email', en: 'Email' },
  'auth.password': { fr: 'Mot de passe', en: 'Password' },
  'auth.confirmPassword': { fr: 'Confirmer le mot de passe', en: 'Confirm password' },
  'auth.displayName': { fr: 'Nom d\'affichage', en: 'Display name' },
  'auth.loginButton': { fr: 'Se connecter', en: 'Sign in' },
  'auth.registerButton': { fr: 'Creer un compte', en: 'Create account' },
  'auth.noAccount': { fr: 'Pas encore de compte ?', en: 'No account yet?' },
  'auth.hasAccount': { fr: 'Deja un compte ?', en: 'Already have an account?' },
  'auth.logout': { fr: 'Deconnexion', en: 'Sign out' },
  'auth.profile': { fr: 'Profil', en: 'Profile' },
  'auth.invalidEmail': { fr: 'Adresse email invalide.', en: 'Invalid email address.' },
  'auth.emailExists': { fr: 'Cet email est deja utilise.', en: 'This email is already in use.' },
  'auth.invalidCredentials': { fr: 'Email ou mot de passe incorrect.', en: 'Invalid email or password.' },
  'auth.passwordTooShort': { fr: 'Le mot de passe doit faire au moins 6 caracteres.', en: 'Password must be at least 6 characters.' },
  'auth.passwordMismatch': { fr: 'Les mots de passe ne correspondent pas.', en: 'Passwords do not match.' },
  'auth.welcome': { fr: 'Bienvenue sur Inkwell', en: 'Welcome to Inkwell' },
  'auth.subtitle': { fr: 'Votre atelier de creation de prompts IA', en: 'Your AI prompt engineering workshop' },
  'auth.changePassword': { fr: 'Changer le mot de passe', en: 'Change password' },
  'auth.currentPassword': { fr: 'Mot de passe actuel', en: 'Current password' },
  'auth.newPassword': { fr: 'Nouveau mot de passe', en: 'New password' },
  'auth.saved': { fr: 'Enregistre !', en: 'Saved!' },
  'auth.save': { fr: 'Enregistrer', en: 'Save' },
  'auth.close': { fr: 'Fermer', en: 'Close' },

  // Analytics
  'tab.analytics': { fr: 'Stats', en: 'Stats' },
  'analytics.title': { fr: 'Statistiques', en: 'Statistics' },
  'analytics.totalExec': { fr: 'Executions totales', en: 'Total executions' },
  'analytics.totalTokens': { fr: 'Tokens totaux', en: 'Total tokens' },
  'analytics.totalCost': { fr: 'Cout total', en: 'Total cost' },
  'analytics.avgLatency': { fr: 'Latence moyenne', en: 'Average latency' },
  'analytics.topModel': { fr: 'Modele le plus utilise', en: 'Most used model' },
  'analytics.perModel': { fr: 'Par modele', en: 'Per model' },
  'analytics.7days': { fr: '7 jours', en: '7 days' },
  'analytics.30days': { fr: '30 jours', en: '30 days' },
  'analytics.allTime': { fr: 'Tout', en: 'All time' },

  // Chain
  'tab.chain': { fr: 'Chain', en: 'Chain' },
  'chain.title': { fr: 'Chainages de prompts', en: 'Prompt chaining' },
  'chain.selectWorkspace': { fr: 'Selectionner un projet...', en: 'Select a project...' },
  'chain.run': { fr: 'Executer la chaine', en: 'Run chain' },
  'chain.running': { fr: 'Execution...', en: 'Running...' },
  'chain.step': { fr: 'Etape', en: 'Step' },
  'chain.noWorkspace': { fr: 'Selectionnez un projet contenant plusieurs prompts.', en: 'Select a project with multiple prompts.' },
  'chain.noPrompts': { fr: 'Ce projet ne contient aucun prompt.', en: 'This project has no prompts.' },

  // Chat
  'tab.chat': { fr: 'Chat', en: 'Chat' },
  'chat.title': { fr: 'Conversation', en: 'Conversation' },
  'chat.systemPrompt': { fr: 'Prompt systeme (optionnel)', en: 'System prompt (optional)' },
  'chat.placeholder': { fr: 'Ecrivez un message...', en: 'Type a message...' },
  'chat.send': { fr: 'Envoyer', en: 'Send' },
  'chat.clear': { fr: 'Effacer la conversation', en: 'Clear conversation' },
  'chat.useCurrentPrompt': { fr: 'Utiliser le prompt actuel comme systeme', en: 'Use current prompt as system' },

  // Misc
  'misc.newPrompt': { fr: 'Nouveau prompt', en: 'New prompt' },
  'misc.error': { fr: 'Erreur', en: 'Error' },
  'misc.unknownError': { fr: 'Erreur inconnue', en: 'Unknown error' },
} as const;

export type TranslationKey = keyof typeof translations;

// Context
export const I18nContext = createContext<Lang>('fr');

// Hook
export function useT() {
  const lang = useContext(I18nContext);
  return function t(key: TranslationKey, replacements?: Record<string, string | number>): string {
    const entry = translations[key];
    let text: string = entry?.[lang] ?? entry?.['fr'] ?? key;
    if (replacements) {
      for (const [k, v] of Object.entries(replacements)) {
        text = text.replace(`{${k}}`, String(v));
      }
    }
    return text;
  };
}
