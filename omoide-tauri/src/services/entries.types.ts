export interface VaultEntryMetadata {
    /**
     * Unique identifier for the password entry in UUID.
     */
    id: string;

    /**
      * Title of entry.
      */
    title: string;

    /**
     * Username field.
     */
    username: string;

    /**
     * URL website field. (optional)
     */
    url?: string;

    /**
     * UNIX timestamp of when entry was created.
     */
    createdAt: number;

    /**
     * UNIX timestamp of when entry was created.
     */
    updatedAt: number;
}