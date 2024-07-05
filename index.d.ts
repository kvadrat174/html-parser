/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export declare function sum(a: number, b: number): number
export interface ReplacedLink {
  href: string
  tracked: string
}
export interface ReplacedUser {
  name?: string
  surname?: string
}
export declare function replaceHandlebarsTokens(buffer: Buffer, user?: ReplacedUser | undefined | null): Buffer
export declare function findAllHrefs(buffer: Buffer): Array<string>
export declare function findHandlebarsTokens(buffer: Buffer, userId?: string | undefined | null): Array<string>
export declare function addPreHeaderAndLinks(buffer: Buffer, links: Array<ReplacedLink>, openLink: string, header?: string | undefined | null, userId?: string | undefined | null): Buffer
