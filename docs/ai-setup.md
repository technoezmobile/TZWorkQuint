# Set up TZ Assistant

Ordinary editing does not require an AI account. To use live inference you need a running compatible endpoint, a model available on that endpoint, and any credentials the endpoint requires. The suite also provides demonstration modes in some editors.

1. Open an editor and find its AI connection or TZ AI Gateway settings. Labels vary by editor. In Wordloom the dialog is **Settings — TZ AI Gateway**.
2. Choose the live gateway/provider mode. If using a local OpenAI-compatible server, start it and load your model first. Enter its actual base URL including the API version path. The default local gateway address in Wordloom is `http://localhost:3001/v1`; the server must be running at that address for it to work.
3. For a remote service enter its documented endpoint and an API key issued for that service. Only enter credentials in the application. A browser chat login does not configure an API connection.
4. Use **Test Connection & Discover Models** where available. Select a model your server actually serves, then save. If discovery fails, check the URL, whether the service is running, authentication, and the service’s models endpoint.
5. Open TZ Assistant with a synthetic document. Select a small range or passage, check the context controls, and send a short request. Read any proposal before applying it. The selected endpoint receives the prompt and the context you include. Hosted inference is subject to the provider’s terms and quota.

If an editor offers **Deterministic Mock**, use it to explore the interface. Switch back to the live provider to test actual inference. Never post API keys, credential-store files, personal documents, or unredacted prompts in an issue.
