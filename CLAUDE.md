# Knowledge Base Parser
This project is a part of the personal task system.
Goal: Provide a context information for other components based on the existing knowledge base
Features:
- Read and parse data from the knowledge base. Low frequency, single-user operation.
- Knowledge base contains: Obsidian Vaults, Scrivener Projects. (It may extends to other types of data)
- Generate context output files that other components can easily read. The output files should be small size so that LLM AI can parse them with low token cost.
