package com.teispace.teistro;

/**
 * What {@code verify.issue} answers: the record's own answer and its receipt
 * ({@code 03-design/receipts-and-datasets.md} §1).
 *
 * @param answer the record's own answer as its JSON reads, {@code {value, provenance}} or the value alone
 * @param receipt the receipt's JSON text, to hand back whole to {@link VerifyArea#receipt(String)}
 */
public record Issued(Object answer, String receipt) {}
